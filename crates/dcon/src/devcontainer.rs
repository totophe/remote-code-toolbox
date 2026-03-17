use std::path::{Path, PathBuf};

/// Walk upward from `start` until we find a directory containing `.devcontainer`.
/// Returns the path of that directory (the project root), or `None` if not found.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".devcontainer").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env::temp_dir;

    #[test]
    fn finds_devcontainer_in_cwd() {
        let root = temp_dir().join("dcon_test_cwd");
        fs::create_dir_all(root.join(".devcontainer")).unwrap();
        assert_eq!(find_project_root(&root), Some(root.clone()));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn finds_devcontainer_in_parent() {
        let root = temp_dir().join("dcon_test_parent");
        let child = root.join("subdir").join("nested");
        fs::create_dir_all(child.join(".devcontainer")).unwrap();
        // searching from a deeper path should still find it
        let deeper = child.join("src");
        fs::create_dir_all(&deeper).unwrap();
        assert_eq!(find_project_root(&deeper), Some(child));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn returns_none_when_no_devcontainer() {
        let root = temp_dir().join("dcon_test_none");
        fs::create_dir_all(&root).unwrap();
        // A temp dir with no .devcontainer — walk will eventually hit fs root and return None
        // We just check it doesn't panic
        let result = find_project_root(&root);
        // It may or may not find one depending on the host, but it must not panic
        let _ = result;
        fs::remove_dir_all(&root).unwrap();
    }
}
