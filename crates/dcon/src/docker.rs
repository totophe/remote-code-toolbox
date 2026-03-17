use std::path::Path;
use std::process::Command;

/// Information about a running dev container.
pub struct Container {
    pub name: String,
}

/// Find the running Docker container associated with the given project root.
///
/// Strategy (in order):
///   1. Match on the `devcontainer.local_folder` label — most reliable.
///   2. Fall back to a name heuristic: the container name contains the
///      last path component of the project root.
///
/// Returns `None` if no matching container is found.
pub fn find(project_root: &Path) -> Result<Container, Error> {
    let output = Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.ID}}\t{{.Names}}\t{{.Label \"devcontainer.local_folder\"}}",
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::DockerNotFound
            } else {
                Error::Io(e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(Error::DockerFailed(stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let project_root_str = project_root.to_string_lossy();
    let project_folder = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Pass 1: label match (exact path)
    for line in stdout.lines() {
        let cols: Vec<&str> = line.splitn(3, '\t').collect();
        if cols.len() < 3 {
            continue;
        }
        let label_value = cols[2].trim();
        if label_value == project_root_str.as_ref() {
            return Ok(Container {
                name: cols[1].to_string(),
            });
        }
    }

    // Pass 2: name heuristic — container name contains the project folder name
    if !project_folder.is_empty() {
        for line in stdout.lines() {
            let cols: Vec<&str> = line.splitn(3, '\t').collect();
            if cols.len() < 2 {
                continue;
            }
            let container_name = cols[1].trim();
            // devcontainers typically produce names like "<folder>-<service>-1"
            if container_name.contains(project_folder) {
                return Ok(Container {
                    name: container_name.to_string(),
                });
            }
        }
    }

    Err(Error::NotRunning)
}

#[derive(Debug)]
pub enum Error {
    DockerNotFound,
    DockerFailed(String),
    NotRunning,
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DockerNotFound => write!(
                f,
                "docker not found — install Docker and make sure it is on your PATH"
            ),
            Error::DockerFailed(msg) => write!(f, "docker ps failed: {msg}"),
            Error::NotRunning => write!(
                f,
                "no running dev container found for this project\nhint: open the project in VS Code first to start the container"
            ),
            Error::Io(e) => write!(f, "i/o error running docker: {e}"),
        }
    }
}
