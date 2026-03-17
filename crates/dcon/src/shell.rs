use std::process::Command;

/// Detect the default shell inside the container.
///
/// Tries `docker exec <container> sh -c 'echo $SHELL'`, then falls back to
/// `/bin/bash`, then `/bin/sh`.
pub fn detect(container: &str) -> String {
    let output = Command::new("docker")
        .args(["exec", container, "sh", "-c", "echo $SHELL"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let shell = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !shell.is_empty() && shell.starts_with('/') {
                return shell;
            }
        }
    }

    // Probe for bash, then fall back to sh
    let has_bash = Command::new("docker")
        .args(["exec", container, "test", "-x", "/bin/bash"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if has_bash {
        "/bin/bash".to_string()
    } else {
        "/bin/sh".to_string()
    }
}
