use bevy::prelude::*;
use std::{fs, path::{Path, PathBuf}, time::SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Scene,
    Texture,
    Mesh,
    Material,
    Audio,
    Script,
    Data,
    Other,
}

#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub path: PathBuf,
    pub kind: AssetKind,
    pub bytes: u64,
    pub modified: Option<SystemTime>,
}

#[derive(Resource, Debug, Default)]
pub struct AssetDatabase {
    pub root: PathBuf,
    pub entries: Vec<AssetEntry>,
    pub generation: u64,
    pub refresh_requested: bool,
}

impl AssetDatabase {
    pub fn scan(&mut self, root: impl Into<PathBuf>) {
        self.root = root.into();
        self.entries.clear();
        let root = self.root.clone();
        visit(&root, &root, 0, 12, 10_000, &mut self.entries);
        self.entries.sort_by(|a, b| a.path.cmp(&b.path));
        self.generation = self.generation.saturating_add(1);
        self.refresh_requested = false;
    }
}

pub fn initial_scan(mut database: ResMut<AssetDatabase>, project: Res<crate::ProjectState>) {
    database.scan(project.root.join("assets"));
}

pub fn refresh_on_request(mut database: ResMut<AssetDatabase>, project: Res<crate::ProjectState>) {
    if database.refresh_requested {
        database.scan(project.root.join("assets"));
    }
}

fn visit(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_files: usize,
    output: &mut Vec<AssetEntry>,
) {
    if depth > max_depth || output.len() >= max_files {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        if output.len() >= max_files {
            return;
        }
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == ".git" || name == "target" || name == ".bevy-gui") {
            continue;
        }
        if path.is_dir() {
            visit(root, &path, depth + 1, max_depth, max_files, output);
            continue;
        }
        let Some(ext) = path.extension().and_then(|v| v.to_str()) else {
            continue;
        };
        let kind = match ext.to_ascii_lowercase().as_str() {
            "scene" | "json" => AssetKind::Scene,
            "png" | "jpg" | "jpeg" | "webp" | "dds" | "ktx2" => AssetKind::Texture,
            "gltf" | "glb" | "obj" | "fbx" => AssetKind::Mesh,
            "material" | "ron" => AssetKind::Material,
            "wav" | "ogg" | "mp3" | "flac" => AssetKind::Audio,
            "rs" | "lua" | "gd" | "wgsl" | "shader" => AssetKind::Script,
            "toml" | "yaml" | "yml" | "csv" => AssetKind::Data,
            _ => AssetKind::Other,
        };
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let meta = fs::metadata(&path).ok();
        output.push(AssetEntry {
            path: relative,
            kind,
            bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            modified: meta.and_then(|m| m.modified().ok()),
        });
    }
}
