use std::path::Path;
use std::process::Command;

/// How to split a newly created window into multiple panes.
pub enum Split {
    /// N panes stacked top-to-bottom (`split-window -v`)
    Stack(u8),
    /// N panes arranged side-by-side (`split-window -h`)
    SideBySide(u8),
}

/// Ensure the tmux session exists, then attach.
///
/// - Creates the session if it doesn't exist, with a shell inside `container`.
/// - If `split` is given and the session's first window has only one pane,
///   splits it into N panes.
/// - `workspace_folder` is passed as `-w` to `docker exec` so the shell starts
///   in the correct directory inside the container.
/// - Attaches with `tmux attach-session`, or `tmux switch-client` if we are
///   already running inside tmux (i.e. `$TMUX` is set).
pub fn connect(
    session: &str,
    container: &str,
    shell: &str,
    project_root: &Path,
    split: Option<Split>,
    workspace_folder: Option<&str>,
    mouse: bool,
) -> Result<(), Error> {
    ensure_session(session, container, shell, project_root, workspace_folder, mouse)?;

    if let Some(ref s) = split {
        apply_split_if_single_pane(session, container, shell, workspace_folder, s)?;
    }

    attach(session)
}

/// Build the `docker exec` command string, optionally with `-w <workspace_folder>`.
fn docker_exec_cmd(container: &str, shell: &str, workspace_folder: Option<&str>) -> String {
    match workspace_folder {
        Some(dir) => format!("docker exec -it -w {dir} {container} {shell}"),
        None => format!("docker exec -it {container} {shell}"),
    }
}

fn ensure_session(
    session: &str,
    container: &str,
    shell: &str,
    project_root: &Path,
    workspace_folder: Option<&str>,
    mouse: bool,
) -> Result<(), Error> {
    let exists = Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::TmuxNotFound
            } else {
                Error::Io(e)
            }
        })?
        .success();

    if !exists {
        let exec_cmd = docker_exec_cmd(container, shell, workspace_folder);
        let status = Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                session,
                "-c",
                &project_root.to_string_lossy(),
                &exec_cmd,
            ])
            .status()
            .map_err(Error::Io)?;

        if !status.success() {
            return Err(Error::TmuxFailed("new-session failed".into()));
        }

        if mouse {
            let _ = Command::new("tmux")
                .args(["set-option", "-g", "mouse", "on"])
                .status();

            // Enable OSC 52 clipboard escape sequence — works over SSH
            // back to the local terminal (iTerm2, WezTerm, Alacritty, etc.).
            let _ = Command::new("tmux")
                .args(["set-option", "-g", "set-clipboard", "on"])
                .status();
            // Allow tmux to pass through OSC 52 to the outer terminal.
            // "all" is needed (not just "on") for mosh compatibility.
            let _ = Command::new("tmux")
                .args(["set-option", "-g", "allow-passthrough", "all"])
                .status();

            // Copy selection to tmux buffer + trigger OSC 52 on mouse drag end.
            for table in &["copy-mode", "copy-mode-vi"] {
                let _ = Command::new("tmux")
                    .args([
                        "bind-key", "-T", table, "MouseDragEnd1Pane",
                        "send", "-X", "copy-selection-and-cancel",
                    ])
                    .status();
            }
        }
    }

    Ok(())
}

/// Split a window into N panes. Called only when the session was just created.
fn apply_split(
    session: &str,
    container: &str,
    shell: &str,
    workspace_folder: Option<&str>,
    split: &Split,
) -> Result<(), Error> {
    let (count, flag, layout) = match split {
        Split::Stack(n) => (*n, "-v", "even-vertical"),
        Split::SideBySide(n) => (*n, "-h", "even-horizontal"),
    };

    let target = format!("{session}:0");
    let exec_cmd = docker_exec_cmd(container, shell, workspace_folder);

    for _ in 1..count {
        let status = Command::new("tmux")
            .args(["split-window", flag, "-t", &target, &exec_cmd])
            .status()
            .map_err(Error::Io)?;

        if !status.success() {
            return Err(Error::TmuxFailed("split-window failed".into()));
        }
    }

    let status = Command::new("tmux")
        .args(["select-layout", "-t", &target, layout])
        .status()
        .map_err(Error::Io)?;

    if !status.success() {
        return Err(Error::TmuxFailed("select-layout failed".into()));
    }

    Ok(())
}

/// Apply split only if the target window currently has a single pane.
fn apply_split_if_single_pane(
    session: &str,
    container: &str,
    shell: &str,
    workspace_folder: Option<&str>,
    split: &Split,
) -> Result<(), Error> {
    let target = format!("{session}:0");

    let output = Command::new("tmux")
        .args(["list-panes", "-t", &target])
        .output()
        .map_err(Error::Io)?;

    let pane_count = String::from_utf8_lossy(&output.stdout).lines().count();

    if pane_count == 1 {
        apply_split(session, container, shell, workspace_folder, split)?;
    }

    Ok(())
}

fn attach(session: &str) -> Result<(), Error> {
    let inside_tmux = std::env::var("TMUX").is_ok();

    let status = if inside_tmux {
        Command::new("tmux")
            .args(["switch-client", "-t", session])
            .status()
            .map_err(Error::Io)?
    } else {
        Command::new("tmux")
            .args(["attach-session", "-t", session])
            .status()
            .map_err(Error::Io)?
    };

    if !status.success() {
        return Err(Error::TmuxFailed("attach/switch failed".into()));
    }
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    TmuxNotFound,
    TmuxFailed(String),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TmuxNotFound => write!(
                f,
                "tmux not found — install tmux and make sure it is on your PATH"
            ),
            Error::TmuxFailed(msg) => write!(f, "tmux error: {msg}"),
            Error::Io(e) => write!(f, "i/o error running tmux: {e}"),
        }
    }
}
