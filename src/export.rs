use std::{fs, io, path::{Path, PathBuf}, process::Command};
use thiserror::Error;

use crate::{project::ProjectState, save_project};

#[derive(Debug, Clone)]
pub struct ExportProfile {
    pub name: String,
    pub output: PathBuf,
    pub include_assets: bool,
    pub include_editor_files: bool,
    pub build_runtime: bool,
}

#[derive(Debug, Clone)]
pub struct ExportReport {
    pub output: PathBuf,
    pub files: usize,
    pub bytes: u64,
    pub executable: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("failed to create output directory: {0}")]
    CreateDirectory(#[source] io::Error),
    #[error("failed to copy file {path}: {source}")]
    Copy { path: PathBuf, source: io::Error },
    #[error("failed to save exported manifest: {0}")]
    Manifest(#[source] crate::ProjectIoError),
    #[error("runtime build failed with status {status}: {stderr}")]
    Build { status: String, stderr: String },
    #[error("failed to start cargo build: {0}")]
    StartBuild(#[source] io::Error),
}

pub fn default_profile(project: &ProjectState) -> ExportProfile {
    ExportProfile {
        name: "desktop".into(),
        output: project.root.join("build").join("desktop"),
        include_assets: true,
        include_editor_files: false,
        build_runtime: true,
    }
}

pub fn export_project(project: &ProjectState, profile: &ExportProfile) -> Result<ExportReport, ExportError> {
    fs::create_dir_all(&profile.output).map_err(ExportError::CreateDirectory)?;
    let mut files = 0usize;
    let mut bytes = 0u64;

    if profile.build_runtime {
        build_runtime(project)?;
    }

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

    let executable = runtime_binary_path(project).filter(|path| path.exists()).map(|source| {
        let destination = profile.output.join(
            source
                .file_name()
                .map(|name| name.to_owned())
                .unwrap_or_else(|| "game".into()),
        );
        let _ = fs::copy(&source, &destination);
        destination
    });
    if executable.is_some() {
        files += 1;
    }

    Ok(ExportReport {
        output: profile.output.clone(),
        files,
        bytes,
        executable,
    })
}

fn build_runtime(project: &ProjectState) -> Result<(), ExportError> {
    let output = Command::new("cargo")
        .current_dir(&project.root)
        .args(["build", "--release"])
        .output()
        .map_err(ExportError::StartBuild)?;
    if !output.status.success() {
        return Err(ExportError::Build {
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(())
}

fn runtime_binary_path(project: &ProjectState) -> Option<PathBuf> {
    let package_name = project
        .name
        .chars()
        .fold(String::new(), |mut out, ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                out.push(ch.to_ascii_lowercase());
            } else if !out.ends_with('_') {
                out.push('_');
            }
            out
        })
        .trim_matches('_')
        .to_owned();
    let package_name = if package_name.is_empty() {
        "bevy_game".to_owned()
    } else {
        package_name
    };
    Some(project.root.join("target").join("release").join(package_name))
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
    let source_path = source.to_path_buf();
    let size = fs::copy(source, destination).map_err(|error| ExportError::Copy {
        path: source_path,
        source: error,
    })?;
    *files += 1;
    *bytes += size;
    Ok(())
}
