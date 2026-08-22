use std::{fs, path::Path};

pub fn scan_assets(root: &Path, max_depth: usize, max_files: usize) -> Vec<String> {
    let mut output = Vec::new();
    visit_assets(root, root, 0, max_depth, max_files, &mut output);
    output.sort();
    output
}

fn visit_assets(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_files: usize,
    output: &mut Vec<String>,
) {
    if depth > max_depth || output.len() >= max_files {
        return;
    }

    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        if output.len() >= max_files {
            break;
        }
        let path = entry.path();
        if path
            .file_name()
            .is_some_and(|name| name == "target" || name == ".git" || name == ".bevy-gui")
        {
            continue;
        }
        if path.is_dir() {
            visit_assets(root, &path, depth + 1, max_depth, max_files, output);
        } else if path.is_file()
            && let Ok(relative) = path.strip_prefix(root)
        {
            output.push(relative.display().to_string());
        }
    }
}
