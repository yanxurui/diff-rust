#[cfg(test)]
mod tests {
    use crate::diff::{compare_directories, build_file_tree};
    use std::path::Path;

    #[test]
    fn test_compare_dirs() {
        let left = Path::new("/tmp/diffr-test/old");
        let right = Path::new("/tmp/diffr-test/new");

        let entries = compare_directories(left, right).unwrap();

        println!("Entries found: {}", entries.len());
        for entry in &entries {
            println!("  {:?}: {} ({:?})", entry.status, entry.path, entry.name);
        }

        let tree = build_file_tree(&entries);
        println!("Tree nodes: {}", tree.len());
        for node in &tree {
            println!("  Node: {} (is_dir: {}, status: {:?})", node.name, node.is_dir, node.status);
        }

        assert!(!entries.is_empty(), "Should find some entries");
    }
}
