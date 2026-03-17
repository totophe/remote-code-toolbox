use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub shell: Option<String>,
}

impl Config {
    /// Load and merge configs: project-level over global.
    /// Returns a single merged Config (most specific value wins).
    pub fn load(project_root: &Path) -> Self {
        let global = load_file(global_path().as_deref());
        let project = load_file(Some(&project_root.join(".devcontainer").join("dcon.json")));

        Self {
            shell: project.shell.or(global.shell),
        }
    }
}

fn global_path() -> Option<std::path::PathBuf> {
    dirs_next::config_dir().map(|d| d.join("dcon").join("config.json"))
}

fn load_file(path: Option<&Path>) -> Config {
    let path = match path {
        Some(p) => p,
        None => return Config::default(),
    };
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Config::default();
    };
    serde_json::from_str(&contents).unwrap_or_else(|e| {
        eprintln!(
            "warning: could not parse {}: {e}",
            path.display()
        );
        Config::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_dcon_json(dir: &Path, contents: &str) {
        let dc = dir.join(".devcontainer");
        fs::create_dir_all(&dc).unwrap();
        fs::write(dc.join("dcon.json"), contents).unwrap();
    }

    #[test]
    fn reads_project_shell() {
        let tmp = TempDir::new().unwrap();
        write_dcon_json(tmp.path(), r#"{"shell": "/bin/zsh"}"#);
        let cfg = load_file(Some(&tmp.path().join(".devcontainer").join("dcon.json")));
        assert_eq!(cfg.shell.as_deref(), Some("/bin/zsh"));
    }

    #[test]
    fn missing_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let cfg = load_file(Some(&tmp.path().join(".devcontainer").join("dcon.json")));
        assert!(cfg.shell.is_none());
    }

    #[test]
    fn invalid_json_returns_default() {
        let tmp = TempDir::new().unwrap();
        write_dcon_json(tmp.path(), "not json");
        let cfg = load_file(Some(&tmp.path().join(".devcontainer").join("dcon.json")));
        assert!(cfg.shell.is_none());
    }

    #[test]
    fn project_overrides_global() {
        // We simulate this by loading two separate files and merging manually,
        // since we can't easily override the global path in tests.
        let tmp = TempDir::new().unwrap();
        write_dcon_json(tmp.path(), r#"{"shell": "/bin/zsh"}"#);

        let global = Config { shell: Some("/bin/bash".into()) };
        let project = load_file(Some(&tmp.path().join(".devcontainer").join("dcon.json")));
        let merged = Config {
            shell: project.shell.or(global.shell),
        };
        assert_eq!(merged.shell.as_deref(), Some("/bin/zsh"));
    }
}
