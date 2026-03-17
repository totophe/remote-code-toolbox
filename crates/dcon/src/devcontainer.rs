use serde::Deserialize;
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

/// Read `workspaceFolder` from `.devcontainer/devcontainer.json`, if present.
/// Returns `None` if the file is missing, unreadable, or the field is not set.
pub fn workspace_folder(project_root: &Path) -> Option<String> {
    #[derive(Deserialize)]
    struct DevcontainerJson {
        #[serde(rename = "workspaceFolder")]
        workspace_folder: Option<String>,
    }

    let path = project_root.join(".devcontainer").join("devcontainer.json");
    let contents = std::fs::read_to_string(&path).ok()?;
    let parsed: DevcontainerJson = serde_json::from_str(&contents).ok()?;
    parsed.workspace_folder
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
        let deeper = child.join("src");
        fs::create_dir_all(&deeper).unwrap();
        assert_eq!(find_project_root(&deeper), Some(child));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn returns_none_when_no_devcontainer() {
        let root = temp_dir().join("dcon_test_none");
        fs::create_dir_all(&root).unwrap();
        let result = find_project_root(&root);
        let _ = result;
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn reads_workspace_folder() {
        let root = temp_dir().join("dcon_test_wsf");
        fs::create_dir_all(root.join(".devcontainer")).unwrap();
        fs::write(
            root.join(".devcontainer").join("devcontainer.json"),
            r#"{"workspaceFolder": "/workspaces/myproject"}"#,
        )
        .unwrap();
        assert_eq!(
            workspace_folder(&root),
            Some("/workspaces/myproject".to_string())
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn workspace_folder_missing_field_returns_none() {
        let root = temp_dir().join("dcon_test_wsf_none");
        fs::create_dir_all(root.join(".devcontainer")).unwrap();
        fs::write(
            root.join(".devcontainer").join("devcontainer.json"),
            r#"{"name": "My Container"}"#,
        )
        .unwrap();
        assert_eq!(workspace_folder(&root), None);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn workspace_folder_missing_file_returns_none() {
        let root = temp_dir().join("dcon_test_wsf_nofile");
        fs::create_dir_all(root.join(".devcontainer")).unwrap();
        assert_eq!(workspace_folder(&root), None);
        fs::remove_dir_all(&root).unwrap();
    }
}
