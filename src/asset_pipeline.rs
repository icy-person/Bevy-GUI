use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, io, path::{Path, PathBuf}, time::SystemTime};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportKind { Image, Mesh, Audio, Shader, Scene, Material, Script, Data, Unknown }
impl ImportKind {
    pub fn from_extension(extension: &str) -> Self { match extension.to_ascii_lowercase().as_str() {
        "png"|"jpg"|"jpeg"|"webp"|"dds"|"ktx2" => Self::Image,
        "gltf"|"glb"|"obj" => Self::Mesh,
        "wav"|"ogg"|"mp3"|"flac" => Self::Audio,
        "wgsl"|"shader" => Self::Shader,
        "scene"|"scene.json"|"prefab.json" => Self::Scene,
        "material"|"mat" => Self::Material,
        "rs"|"lua"|"gd" => Self::Script,
        "ron"|"toml"|"yaml"|"yml"|"json"|"csv" => Self::Data,
        _ => Self::Unknown,
    }}
    pub fn display_name(self) -> &'static str { match self { Self::Image=>"Image",Self::Mesh=>"Mesh",Self::Audio=>"Audio",Self::Shader=>"Shader",Self::Scene=>"Scene",Self::Material=>"Material",Self::Script=>"Script",Self::Data=>"Data",Self::Unknown=>"Unknown" } }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)] pub enum ImportStatus { Pending, Imported, Failed, Unsupported }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct ImportedAsset { pub source:PathBuf,pub generated:Vec<PathBuf>,pub kind:ImportKind,pub status:ImportStatus,pub source_bytes:u64,pub modified_unix_ms:Option<u128>,pub importer_version:u32,pub error:Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct ImportSettings { pub copy_sources:bool,pub generate_metadata:bool,pub preserve_directories:bool,pub fail_on_unknown:bool }
impl Default for ImportSettings { fn default()->Self{Self{copy_sources:false,generate_metadata:true,preserve_directories:true,fail_on_unknown:false}} }
#[derive(Resource,Debug,Clone,Serialize,Deserialize,Default)] pub struct ImportDatabase { pub version:u32,pub project_root:PathBuf,pub imported_root:PathBuf,pub settings:ImportSettings,pub assets:BTreeMap<String,ImportedAsset>,pub generation:u64 }
impl ImportDatabase {
    pub const VERSION:u32=1; pub const IMPORTER_VERSION:u32=1;
    pub fn new(project_root:impl Into<PathBuf>)->Self{let project_root=project_root.into();Self{version:Self::VERSION,imported_root:project_root.join(".bevy-gui").join("imported"),project_root,settings:ImportSettings::default(),assets:BTreeMap::new(),generation:0}}
    pub fn scan_assets(&mut self)->Vec<PathBuf>{let root=self.project_root.join("assets");let mut files=Vec::new();visit_assets(&root,&mut files,0,32,100000);files.sort();files}
    pub fn import_all(&mut self)->ImportReport{let files=self.scan_assets();let mut report=ImportReport::default();for file in files{match self.import_file(&file){Ok(asset)=>{if asset.status==ImportStatus::Imported{report.imported+=1}else if asset.status==ImportStatus::Unsupported{report.unsupported+=1}self.assets.insert(normalize_key(&file),asset);},Err(error)=>{report.failed+=1;report.errors.push(error.to_string());}}}self.generation=self.generation.saturating_add(1);report.generation=self.generation;report}
    pub fn import_file(&self,source:&Path)->Result<ImportedAsset,ImportError>{let relative=source.strip_prefix(self.project_root.join("assets")).unwrap_or(source).to_path_buf();let metadata=fs::metadata(source).map_err(|e|ImportError::Read{path:source.to_path_buf(),source:e})?;let kind=source.extension().and_then(|e|e.to_str()).map(ImportKind::from_extension).unwrap_or(ImportKind::Unknown);if kind==ImportKind::Unknown&&self.settings.fail_on_unknown{return Err(ImportError::Unsupported(source.to_path_buf()))};let modified_unix_ms=metadata.modified().ok().and_then(|t|t.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d|d.as_millis());let mut asset=ImportedAsset{source:relative.clone(),generated:Vec::new(),kind,status:if kind==ImportKind::Unknown{ImportStatus::Unsupported}else{ImportStatus::Pending},source_bytes:metadata.len(),modified_unix_ms,importer_version:Self::IMPORTER_VERSION,error:None};if kind==ImportKind::Unknown{return Ok(asset)}let destination=self.imported_root.join(&relative);if self.settings.copy_sources{if let Some(parent)=destination.parent(){fs::create_dir_all(parent).map_err(ImportError::CreateDirectory)?;}fs::copy(source,&destination).map_err(|source_error|ImportError::Copy{path:source.to_path_buf(),source:source_error})?;asset.generated.push(destination)}if self.settings.generate_metadata{let metadata_path=metadata_path(&self.imported_root,&relative);if let Some(parent)=metadata_path.parent(){fs::create_dir_all(parent).map_err(ImportError::CreateDirectory)?;}let json=serde_json::to_string_pretty(&asset).map_err(ImportError::Serialize)?;fs::write(&metadata_path,json).map_err(ImportError::Write)?;asset.generated.push(metadata_path)}asset.status=ImportStatus::Imported;Ok(asset)}
    pub fn remove_missing(&mut self)->usize{let current:BTreeSet<_>=self.scan_assets().into_iter().map(|p|normalize_key(&p)).collect();let before=self.assets.len();self.assets.retain(|key,_|current.contains(key));before.saturating_sub(self.assets.len())}
    pub fn asset(&self,relative:&Path)->Option<&ImportedAsset>{self.assets.get(&normalize_key(relative))}
    pub fn save(&self)->Result<(),ImportError>{let path=self.project_root.join(".bevy-gui").join("imports.json");if let Some(parent)=path.parent(){fs::create_dir_all(parent).map_err(ImportError::CreateDirectory)?;}let json=serde_json::to_string_pretty(self).map_err(ImportError::Serialize)?;fs::write(path,json).map_err(ImportError::Write)}
    pub fn load(project_root:impl Into<PathBuf>)->Result<Self,ImportError>{let project_root=project_root.into();let path=project_root.join(".bevy-gui").join("imports.json");if !path.exists(){return Ok(Self::new(project_root))}let json=fs::read_to_string(path).map_err(ImportError::DatabaseRead)?;let mut db:Self=serde_json::from_str(&json).map_err(ImportError::DatabaseParse)?;db.project_root=project_root.clone();db.imported_root=project_root.join(".bevy-gui").join("imported");Ok(db)}
}
#[derive(Debug,Default)] pub struct ImportReport { pub imported:usize,pub failed:usize,pub unsupported:usize,pub generation:u64,pub errors:Vec<String> }
#[derive(Debug,Error)] pub enum ImportError { #[error("failed to read asset {path}: {source}")] Read{path:PathBuf,source:io::Error}, #[error("unsupported asset {0}")] Unsupported(PathBuf), #[error("failed to create import directory: {0}")] CreateDirectory(#[source]io::Error), #[error("failed to copy asset {path}: {source}")] Copy{path:PathBuf,source:io::Error}, #[error("failed to serialize metadata: {0}")] Serialize(#[source]serde_json::Error), #[error("failed to write metadata: {0}")] Write(#[source]io::Error), #[error("failed to read import database: {0}")] DatabaseRead(#[source]io::Error), #[error("failed to parse import database: {0}")] DatabaseParse(#[source]serde_json::Error) }
pub fn metadata_path(root:&Path,relative:&Path)->PathBuf{let mut value=root.join(relative);let extension=value.extension().and_then(|v|v.to_str()).unwrap_or("asset");let filename=value.file_stem().and_then(|v|v.to_str()).unwrap_or("asset");value.set_file_name(format!("{filename}.{extension}.import.json"));value}
pub fn normalize_key(path:&Path)->String{path.to_string_lossy().replace('\\',"/")}
fn visit_assets(current:&Path,output:&mut Vec<PathBuf>,depth:usize,max_depth:usize,max_files:usize){if depth>max_depth||output.len()>=max_files||!current.exists(){return};let Ok(entries)=fs::read_dir(current)else{return};for entry in entries.flatten(){if output.len()>=max_files{return};let path=entry.path();let hidden=path.file_name().and_then(|v|v.to_str()).is_some_and(|n|n.starts_with('.'));if hidden{continue}if path.is_dir(){visit_assets(&path,output,depth+1,max_depth,max_files)}else if path.is_file(){output.push(path)}}}
#[cfg(test)] mod tests { use super::*; #[test] fn extension_classification_is_stable(){assert_eq!(ImportKind::from_extension("png"),ImportKind::Image);assert_eq!(ImportKind::from_extension("GLB"),ImportKind::Mesh);assert_eq!(ImportKind::from_extension("wgsl"),ImportKind::Shader);assert_eq!(ImportKind::from_extension("unknown"),ImportKind::Unknown)} #[test] fn metadata_path_preserves_source_extension(){let path=metadata_path(Path::new(".imports"),Path::new("textures/player.png"));assert_eq!(path,PathBuf::from(".imports/textures/player.png.import.json"));} }
