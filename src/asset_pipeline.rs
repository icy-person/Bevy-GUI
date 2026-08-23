use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, io, path::{Path, PathBuf}, time::SystemTime};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportKind { Image, Mesh, Audio, Shader, Scene, Material, Script, Data, Unknown }

impl ImportKind {
    pub fn from_path(path: &Path) -> Self {
        let file_name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default().to_ascii_lowercase();
        if file_name.ends_with(".scene.json") || file_name.ends_with(".prefab.json") { return Self::Scene; }
        let extension = path.extension().and_then(|v| v.to_str()).unwrap_or_default();
        match extension.to_ascii_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "dds" | "ktx2" => Self::Image,
            "gltf" | "glb" | "obj" => Self::Mesh,
            "wav" | "ogg" | "mp3" | "flac" => Self::Audio,
            "wgsl" | "shader" => Self::Shader,
            "scene" | "prefab" => Self::Scene,
            "material" | "mat" => Self::Material,
            "rs" | "lua" | "gd" => Self::Script,
            "ron" | "toml" | "yaml" | "yml" | "json" | "csv" => Self::Data,
            _ => Self::Unknown,
        }
    }

    pub fn from_extension(extension: &str) -> Self {
        Self::from_path(Path::new(&format!("asset.{extension}")))
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Image => "Image", Self::Mesh => "Mesh", Self::Audio => "Audio",
            Self::Shader => "Shader", Self::Scene => "Scene", Self::Material => "Material",
            Self::Script => "Script", Self::Data => "Data", Self::Unknown => "Unknown",
        }
    }

    pub fn is_supported(self) -> bool { self != Self::Unknown }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportStatus { Pending, Imported, Failed, Unsupported }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedAsset {
    pub source: PathBuf,
    pub generated: Vec<PathBuf>,
    pub kind: ImportKind,
    pub status: ImportStatus,
    pub source_bytes: u64,
    pub modified_unix_ms: Option<u128>,
    pub importer_version: u32,
    pub content_fingerprint: Option<String>,
    pub error: Option<String>,
}

impl ImportedAsset {
    pub fn is_stale(&self, bytes: u64, modified_unix_ms: Option<u128>) -> bool {
        self.source_bytes != bytes || self.modified_unix_ms != modified_unix_ms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    pub copy_sources: bool,
    pub generate_metadata: bool,
    pub preserve_directories: bool,
    pub fail_on_unknown: bool,
    pub remove_missing_entries: bool,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            copy_sources: false,
            generate_metadata: true,
            preserve_directories: true,
            fail_on_unknown: false,
            remove_missing_entries: true,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportDatabase {
    pub version: u32,
    pub project_root: PathBuf,
    pub imported_root: PathBuf,
    pub settings: ImportSettings,
    pub assets: BTreeMap<String, ImportedAsset>,
    pub generation: u64,
}

impl ImportDatabase {
    pub const VERSION: u32 = 2;
    pub const IMPORTER_VERSION: u32 = 2;

    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self {
            version: Self::VERSION,
            imported_root: project_root.join(".bevy-gui").join("imported"),
            project_root,
            settings: ImportSettings::default(),
            assets: BTreeMap::new(),
            generation: 0,
        }
    }

    pub fn assets_root(&self) -> PathBuf { self.project_root.join("assets") }
    pub fn database_path(&self) -> PathBuf { self.project_root.join(".bevy-gui").join("imports.json") }

    pub fn scan_assets(&self) -> Vec<PathBuf> {
        let root = self.assets_root();
        let mut files = Vec::new();
        visit_assets(&root, &mut files, 0, 32, 100_000);
        files.sort();
        files
    }

    pub fn import_all(&mut self) -> ImportReport {
        let files = self.scan_assets();
        let mut report = ImportReport::default();
        let mut seen = BTreeSet::new();
        for file in files {
            let key = normalize_key(file.strip_prefix(self.assets_root()).unwrap_or(&file));
            seen.insert(key.clone());
            match self.import_file(&file) {
                Ok(asset) => {
                    match asset.status {
                        ImportStatus::Imported => report.imported += 1,
                        ImportStatus::Unsupported => report.unsupported += 1,
                        ImportStatus::Failed => report.failed += 1,
                        ImportStatus::Pending => {}
                    }
                    self.assets.insert(key, asset);
                }
                Err(error) => {
                    report.failed += 1;
                    report.errors.push(error.to_string());
                }
            }
        }
        if self.settings.remove_missing_entries {
            let before = self.assets.len();
            self.assets.retain(|key, _| seen.contains(key));
            report.removed = before.saturating_sub(self.assets.len());
        }
        self.generation = self.generation.saturating_add(1);
        report.generation = self.generation;
        report
    }

    pub fn import_file(&self, source: &Path) -> Result<ImportedAsset, ImportError> {
        let assets_root = self.assets_root();
        let relative = source.strip_prefix(&assets_root).unwrap_or(source).to_path_buf();
        let metadata = fs::metadata(source).map_err(|e| ImportError::Read { path: source.to_path_buf(), source: e })?;
        let kind = ImportKind::from_path(source);
        if !kind.is_supported() && self.settings.fail_on_unknown {
            return Err(ImportError::Unsupported(source.to_path_buf()));
        }
        let modified_unix_ms = metadata.modified().ok().and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|duration| duration.as_millis());
        let fingerprint = fingerprint(&metadata, modified_unix_ms);
        let mut asset = ImportedAsset {
            source: relative.clone(),
            generated: Vec::new(),
            kind,
            status: if kind.is_supported() { ImportStatus::Pending } else { ImportStatus::Unsupported },
            source_bytes: metadata.len(),
            modified_unix_ms,
            importer_version: Self::IMPORTER_VERSION,
            content_fingerprint: fingerprint,
            error: None,
        };
        if !kind.is_supported() { return Ok(asset); }

        let destination = if self.settings.preserve_directories {
            self.imported_root.join(&relative)
        } else {
            self.imported_root.join(relative.file_name().unwrap_or_else(|| std::ffi::OsStr::new("asset")))
        };

        if self.settings.copy_sources {
            ensure_parent(&destination)?;
            fs::copy(source, &destination).map_err(|error| ImportError::Copy { path: source.to_path_buf(), source: error })?;
            asset.generated.push(destination.clone());
        }

        if self.settings.generate_metadata {
            let metadata_path = metadata_path(&self.imported_root, &relative);
            ensure_parent(&metadata_path)?;
            let json = serde_json::to_string_pretty(&asset).map_err(ImportError::Serialize)?;
            fs::write(&metadata_path, json).map_err(ImportError::Write)?;
            asset.generated.push(metadata_path);
        }

        asset.status = ImportStatus::Imported;
        Ok(asset)
    }

    pub fn remove_missing(&mut self) -> usize {
        let current: BTreeSet<_> = self.scan_assets().into_iter().map(|path| normalize_key(path.strip_prefix(self.assets_root()).unwrap_or(&path))).collect();
        let before = self.assets.len();
        self.assets.retain(|key, _| current.contains(key));
        before.saturating_sub(self.assets.len())
    }

    pub fn asset(&self, relative: &Path) -> Option<&ImportedAsset> {
        self.assets.get(&normalize_key(relative))
    }

    pub fn save(&self) -> Result<(), ImportError> {
        let path = self.database_path();
        ensure_parent(&path)?;
        let json = serde_json::to_string_pretty(self).map_err(ImportError::Serialize)?;
        fs::write(path, json).map_err(ImportError::Write)
    }

    pub fn load(project_root: impl Into<PathBuf>) -> Result<Self, ImportError> {
        let project_root = project_root.into();
        let path = project_root.join(".bevy-gui").join("imports.json");
        if !path.exists() { return Ok(Self::new(project_root)); }
        let json = fs::read_to_string(path).map_err(ImportError::DatabaseRead)?;
        let mut database: Self = serde_json::from_str(&json).map_err(ImportError::DatabaseParse)?;
        database.version = Self::VERSION;
        database.project_root = project_root.clone();
        database.imported_root = project_root.join(".bevy-gui").join("imported");
        Ok(database)
    }
}

#[derive(Debug, Default)]
pub struct ImportReport {
    pub imported: usize,
    pub failed: usize,
    pub unsupported: usize,
    pub removed: usize,
    pub generation: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("failed to read asset {path}: {source}")] Read { path: PathBuf, source: io::Error },
    #[error("unsupported asset {0}")] Unsupported(PathBuf),
    #[error("failed to create import directory: {0}")] CreateDirectory(#[source] io::Error),
    #[error("failed to copy asset {path}: {source}")] Copy { path: PathBuf, source: io::Error },
    #[error("failed to serialize metadata: {0}")] Serialize(#[source] serde_json::Error),
    #[error("failed to write metadata: {0}")] Write(#[source] io::Error),
    #[error("failed to read import database: {0}")] DatabaseRead(#[source] io::Error),
    #[error("failed to parse import database: {0}")] DatabaseParse(#[source] serde_json::Error),
}

fn ensure_parent(path: &Path) -> Result<(), ImportError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ImportError::CreateDirectory)?;
    }
    Ok(())
}

fn fingerprint(metadata: &fs::Metadata, modified_unix_ms: Option<u128>) -> Option<String> {
    Some(format!("{}:{}:{}", metadata.len(), modified_unix_ms.unwrap_or_default(), metadata.is_file()))
}

pub fn metadata_path(root: &Path, relative: &Path) -> PathBuf {
    let mut value = root.join(relative);
    let extension = value.extension().and_then(|v| v.to_str()).unwrap_or("asset");
    let filename = value.file_stem().and_then(|v| v.to_str()).unwrap_or("asset");
    value.set_file_name(format!("{filename}.{extension}.import.json"));
    value
}

pub fn normalize_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn visit_assets(current: &Path, output: &mut Vec<PathBuf>, depth: usize, max_depth: usize, max_files: usize) {
    if depth > max_depth || output.len() >= max_files || !current.exists() { return; }
    let Ok(entries) = fs::read_dir(current) else { return; };
    for entry in entries.flatten() {
        if output.len() >= max_files { return; }
        let path = entry.path();
        let hidden = path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.starts_with('.'));
        if hidden { continue; }
        if path.is_dir() { visit_assets(&path, output, depth + 1, max_depth, max_files); }
        else if path.is_file() { output.push(path); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_classification_is_stable() {
        assert_eq!(ImportKind::from_extension("png"), ImportKind::Image);
        assert_eq!(ImportKind::from_extension("GLB"), ImportKind::Mesh);
        assert_eq!(ImportKind::from_extension("wgsl"), ImportKind::Shader);
        assert_eq!(ImportKind::from_path(Path::new("main.scene.json")), ImportKind::Scene);
        assert_eq!(ImportKind::from_extension("unknown"), ImportKind::Unknown);
    }

    #[test]
    fn metadata_path_preserves_source_extension() {
        let path = metadata_path(Path::new(".imports"), Path::new("textures/player.png"));
        assert_eq!(path, PathBuf::from(".imports/textures/player.png.import.json"));
    }
}
