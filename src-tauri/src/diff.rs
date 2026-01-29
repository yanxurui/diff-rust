use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum DiffError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("Path error: {0}")]
    Path(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub status: FileStatus,
    pub is_dir: bool,
    pub left_path: Option<String>,
    pub right_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub status: Option<FileStatus>,
    pub is_dir: bool,
    pub children: Vec<FileTreeNode>,
    pub left_path: Option<String>,
    pub right_path: Option<String>,
}

pub fn compare_directories(
    left_dir: &Path,
    right_dir: &Path,
) -> Result<Vec<FileEntry>, DiffError> {
    let mut left_files: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut right_files: HashMap<PathBuf, PathBuf> = HashMap::new();

    // Walk left directory
    for entry in WalkDir::new(left_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let relative = entry
            .path()
            .strip_prefix(left_dir)
            .map_err(|e| DiffError::Path(e.to_string()))?;
        left_files.insert(relative.to_path_buf(), entry.path().to_path_buf());
    }

    // Walk right directory
    for entry in WalkDir::new(right_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let relative = entry
            .path()
            .strip_prefix(right_dir)
            .map_err(|e| DiffError::Path(e.to_string()))?;
        right_files.insert(relative.to_path_buf(), entry.path().to_path_buf());
    }

    let mut entries = Vec::new();

    // Collect deleted and added files for rename detection
    let mut deleted_files: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut added_files: Vec<(PathBuf, PathBuf)> = Vec::new();

    // Find files that exist in both directories (modified or unchanged)
    for (relative, right_path) in &right_files {
        if let Some(left_path) = left_files.get(relative) {
            // File exists in both - check if modified
            let status = if files_differ(left_path, right_path)? {
                FileStatus::Modified
            } else {
                FileStatus::Unchanged
            };

            let name = relative
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            entries.push(FileEntry {
                path: relative.to_string_lossy().to_string(),
                name,
                status,
                is_dir: false,
                left_path: Some(left_path.to_string_lossy().to_string()),
                right_path: Some(right_path.to_string_lossy().to_string()),
            });
        } else {
            // File only in right - potentially added or renamed
            added_files.push((relative.clone(), right_path.clone()));
        }
    }

    // Find deleted files (in left but not in right)
    for (relative, left_path) in &left_files {
        if !right_files.contains_key(relative) {
            deleted_files.push((relative.clone(), left_path.clone()));
        }
    }

    // Detect renames: match deleted files with added files by content
    let mut renamed_left: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut renamed_right: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for (deleted_rel, deleted_path) in &deleted_files {
        for (added_rel, added_path) in &added_files {
            if renamed_right.contains(added_rel) {
                continue;
            }

            // Check if files have identical content
            if !files_differ(deleted_path, added_path)? {
                // Found a rename!
                let name = added_rel
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                entries.push(FileEntry {
                    path: format!("{} → {}", deleted_rel.to_string_lossy(), added_rel.to_string_lossy()),
                    name,
                    status: FileStatus::Renamed,
                    is_dir: false,
                    left_path: Some(deleted_path.to_string_lossy().to_string()),
                    right_path: Some(added_path.to_string_lossy().to_string()),
                });

                renamed_left.insert(deleted_rel.clone());
                renamed_right.insert(added_rel.clone());
                break;
            }
        }
    }

    // Add remaining deleted files (not renamed)
    for (relative, left_path) in &deleted_files {
        if !renamed_left.contains(relative) {
            let name = relative
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            entries.push(FileEntry {
                path: relative.to_string_lossy().to_string(),
                name,
                status: FileStatus::Deleted,
                is_dir: false,
                left_path: Some(left_path.to_string_lossy().to_string()),
                right_path: None,
            });
        }
    }

    // Add remaining added files (not renamed)
    for (relative, right_path) in &added_files {
        if !renamed_right.contains(relative) {
            let name = relative
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            entries.push(FileEntry {
                path: relative.to_string_lossy().to_string(),
                name,
                status: FileStatus::Added,
                is_dir: false,
                left_path: None,
                right_path: Some(right_path.to_string_lossy().to_string()),
            });
        }
    }

    // Sort by path
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

fn files_differ(left: &Path, right: &Path) -> Result<bool, DiffError> {
    let left_content = std::fs::read(left)?;
    let right_content = std::fs::read(right)?;
    Ok(left_content != right_content)
}

pub fn build_file_tree(entries: &[FileEntry]) -> Vec<FileTreeNode> {
    let mut root_children: Vec<FileTreeNode> = Vec::new();

    for entry in entries {
        // Skip unchanged files
        if entry.status == FileStatus::Unchanged {
            continue;
        }

        // For renamed files, use the NEW path (after →) for tree placement
        let tree_path = if entry.status == FileStatus::Renamed {
            if let Some(arrow_pos) = entry.path.find(" → ") {
                &entry.path[arrow_pos + " → ".len()..]
            } else {
                &entry.path
            }
        } else {
            &entry.path
        };

        let parts: Vec<&str> = tree_path.split('/').collect();
        insert_into_tree(&mut root_children, &parts, entry);
    }

    // Sort children recursively
    sort_tree(&mut root_children);

    root_children
}

fn insert_into_tree(nodes: &mut Vec<FileTreeNode>, parts: &[&str], entry: &FileEntry) {
    if parts.is_empty() {
        return;
    }

    let name = parts[0];
    let is_leaf = parts.len() == 1;

    // Find or create the node
    let node_idx = nodes.iter().position(|n| n.name == name);

    if let Some(idx) = node_idx {
        if !is_leaf {
            insert_into_tree(&mut nodes[idx].children, &parts[1..], entry);
        }
    } else {
        let mut new_node = if is_leaf {
            FileTreeNode {
                name: name.to_string(),
                path: entry.path.clone(),
                status: Some(entry.status.clone()),
                is_dir: false,
                children: Vec::new(),
                left_path: entry.left_path.clone(),
                right_path: entry.right_path.clone(),
            }
        } else {
            // Build path for directory
            let dir_path = parts[0..1].join("/");
            FileTreeNode {
                name: name.to_string(),
                path: dir_path,
                status: None,
                is_dir: true,
                children: Vec::new(),
                left_path: None,
                right_path: None,
            }
        };

        if !is_leaf {
            insert_into_tree(&mut new_node.children, &parts[1..], entry);
        }

        nodes.push(new_node);
    }
}

fn sort_tree(nodes: &mut [FileTreeNode]) {
    // Directories first, then alphabetically
    nodes.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    for node in nodes.iter_mut() {
        sort_tree(&mut node.children);
    }
}
