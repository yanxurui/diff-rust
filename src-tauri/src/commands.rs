use crate::delta::{generate_diff, get_file_content, DiffOptions, DiffResult};
use crate::diff::{build_file_tree, compare_directories, FileEntry, FileTreeNode};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeResult {
    pub tree: Vec<FileTreeNode>,
    pub files: Vec<FileEntry>,
    pub total_changes: usize,
    pub added: usize,
    pub deleted: usize,
    pub modified: usize,
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn get_file_tree(leftDir: &str, rightDir: &str) -> Result<FileTreeResult, String> {
    let left_path = Path::new(leftDir);
    let right_path = Path::new(rightDir);

    if !left_path.exists() {
        return Err(format!("Left directory does not exist: {}", leftDir));
    }

    if !right_path.exists() {
        return Err(format!("Right directory does not exist: {}", rightDir));
    }

    let entries = compare_directories(left_path, right_path).map_err(|e| e.to_string())?;

    let tree = build_file_tree(&entries);

    // Count changes by status
    let added = entries
        .iter()
        .filter(|e| matches!(e.status, crate::diff::FileStatus::Added))
        .count();
    let deleted = entries
        .iter()
        .filter(|e| matches!(e.status, crate::diff::FileStatus::Deleted))
        .count();
    let modified = entries
        .iter()
        .filter(|e| matches!(e.status, crate::diff::FileStatus::Modified))
        .count();

    Ok(FileTreeResult {
        tree,
        files: entries
            .into_iter()
            .filter(|e| !matches!(e.status, crate::diff::FileStatus::Unchanged))
            .collect(),
        total_changes: added + deleted + modified,
        added,
        deleted,
        modified,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn get_diff(
    leftPath: Option<&str>,
    rightPath: Option<&str>,
    options: DiffOptions,
) -> Result<DiffResult, String> {
    let left = leftPath.map(Path::new);
    let right = rightPath.map(Path::new);

    generate_diff(left, right, &options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_file_content(path: &str) -> Result<String, String> {
    get_file_content(Path::new(path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_delta() -> bool {
    crate::delta::check_delta_installed()
}

#[tauri::command]
pub fn get_app_args() -> Vec<String> {
    std::env::args().collect()
}
