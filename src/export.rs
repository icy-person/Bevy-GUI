use std::{fs, io, path::{Path, PathBuf}};
use thiserror::Error;

use crate::{project::ProjectState, save_project};

#[derive(Debug, Clone)]
pub struct ExportProfile {
    pub name: String,
    pub output: PathBuf,
    pub include_assets: bool,
    pub include_editor_files: bool,
}

#[derive(Debug, Clone)]
pub struct ExportReport {
    pub output: PathBuf,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("failed to create output directory: {0}")]
    CreateDirectory(#[source] io::Error),
    #[error("failed to copy file {path}: {source}")]
    Copy { path: PathBuf, source: io::Error },
    #[error("failed to save exported manifest: {0}")]
    Manifest(#[source] crate::ProjectIoError),
}

pub fn default_profile(project: &ProjectState) -> ExportProfile {
    ExportProfile {
        name: "desktop".into(),
        output: project.root.join("build").join("desktop"),
        include_assets: true,
        include_editor_files: false,
    }
}

pub fn export_project(project: &ProjectState, profile: &ExportProfile) -> Result<ExportReport, ExportError> {
    fs::create_dir_all(&profile.output).map_err(ExportError::CreateDirectory)?;
    let mut files = 0usize;
    let mut bytes = 0u64;

    let manifest_root = profile.output.join("project.godot-rs.json");
    let mut exported = project.clone();
    exported.root = profile.output.clone();
    exported.dirty = false;
    save_project(&profile.output, &exported).map_err(ExportError::Manifest)?;
    files += 1;

    if let Some(scene) = &project.main_scene {
        let source = project.root.join(scene);
        let destination = profile.output.join(scene);
        copy_file(&source, &destination, &mut files, &mut bytes)?;
    }

    if profile.include_assets {
        let source = project.root.join("assets");
        if source.exists() {
            copy_tree(&source, &profile.output.join("assets"), &mut files, &mut bytes)?;
        }
    }

    let _ = manifest_root;
    Ok(ExportReport {
        output: profile.output.clone(),
        files,
        bytes,
    })
}

fn copy_tree(source: &Path, destination: &Path, files: &mut usize, bytes: &mut u64) -> Result<(), ExportError> {
    fs::create_dir_all(destination).map_err(ExportError::CreateDirectory)?;
    for entry in fs::read_dir(source).map_err(ExportError::CreateDirectory)? {
        let entry = entry.map_err(ExportError::CreateDirectory)?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to, files, bytes)?;
        } else if from.is_file() {
            copy_file(&from, &to, files, bytes)?;
        }
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path, files: &mut usize, bytes: &mut u64) -> Result<(), ExportError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(ExportError::CreateDirectory)?;
    }
    let size = fs::copy(source, destination).map_err(|source| ExportError::Copy {
        path: source.to_path_buf(),
        source,
    })?;
    *files += 1;
    *bytes += size;
    Ok(())
}
