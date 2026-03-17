use std::path::Path;
use std::process::Command;

/// How to split a newly created window into multiple panes.
pub enum Split {
    /// N panes stacked top-to-bottom (`split-window -v`)
    Stack(u8),
    /// N panes arranged side-by-side (`split-window -h`)
    SideBySide(u8),
}

/// Ensure the tmux session exists and the right window is selected, then attach.
///
/// - Creates the session if it doesn't exist, with a shell inside `container`.
/// - If `window` is given, creates the named window if needed, then selects it.
/// - If `split` is given and the window was just created, splits it into N panes
///   with `tmux select-layout even-vertical` / `even-horizontal` afterwards.
/// - Attaches with `tmux attach-session`, or `tmux switch-client` if we are
///   already running inside tmux (i.e. `$TMUX` is set).
pub fn connect(
    session: &str,
    window: Option<&str>,
    container: &str,
    shell: &str,
    project_root: &Path,
    split: Option<Split>,
) -> Result<(), Error> {
    ensure_session(session, container, shell, project_root)?;

    // Determine the tmux target for split operations.
    let window_target = if let Some(name) = window {
        let created = ensure_window(session, name, container, shell)?;
        select_window(session, name)?;
        if created {
            // Apply split to the newly created named window.
            if let Some(ref s) = split {
                apply_split(session, Some(name), container, shell, s)?;
            }
        }
        format!("{session}:{name}")
    } else {
        // No named window — apply split to window 0 only when the session was
        // just created (ensure_session returns whether it was new).
        // We re-check by trying to get the pane count; simpler to just always
        // apply split on the default window when requested and the session is fresh.
        // Since ensure_session is idempotent, we track freshness via a second approach:
        // apply split only if the window has exactly 1 pane right now.
        if let Some(ref s) = split {
            apply_split_if_single_pane(session, None, container, shell, s)?;
        }
        format!("{session}:0")
    };

    let _ = window_target;
    attach(session)
}

fn ensure_session(
    session: &str,
    container: &str,
    shell: &str,
    project_root: &Path,
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
        let exec_cmd = format!("docker exec -it {container} {shell}");
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
    }

    Ok(())
}

/// Returns `true` if the window was newly created, `false` if it already existed.
fn ensure_window(session: &str, name: &str, container: &str, shell: &str) -> Result<bool, Error> {
    let target = format!("{session}:{name}");
    let exists = Command::new("tmux")
        .args(["select-window", "-t", &target])
        .status()
        .map_err(Error::Io)?
        .success();

    if exists {
        return Ok(false);
    }

    let exec_cmd = format!("docker exec -it {container} {shell}");
    let status = Command::new("tmux")
        .args(["new-window", "-t", session, "-n", name, &exec_cmd])
        .status()
        .map_err(Error::Io)?;

    if !status.success() {
        return Err(Error::TmuxFailed(format!("new-window '{name}' failed")));
    }
    Ok(true)
}

fn select_window(session: &str, name: &str) -> Result<(), Error> {
    let target = format!("{session}:{name}");
    let status = Command::new("tmux")
        .args(["select-window", "-t", &target])
        .status()
        .map_err(Error::Io)?;

    if !status.success() {
        return Err(Error::TmuxFailed(format!("select-window '{name}' failed")));
    }
    Ok(())
}

/// Split a window into N panes. Called only when the window was just created.
fn apply_split(
    session: &str,
    window: Option<&str>,
    container: &str,
    shell: &str,
    split: &Split,
) -> Result<(), Error> {
    let (count, flag, layout) = match split {
        Split::Stack(n) => (*n, "-v", "even-vertical"),
        Split::SideBySide(n) => (*n, "-h", "even-horizontal"),
    };

    let target = match window {
        Some(name) => format!("{session}:{name}"),
        None => format!("{session}:0"),
    };

    let exec_cmd = format!("docker exec -it {container} {shell}");

    // The window already has 1 pane; create (count - 1) more.
    for _ in 1..count {
        let status = Command::new("tmux")
            .args(["split-window", flag, "-t", &target, &exec_cmd])
            .status()
            .map_err(Error::Io)?;

        if !status.success() {
            return Err(Error::TmuxFailed("split-window failed".into()));
        }
    }

    // Even out the pane sizes.
    let status = Command::new("tmux")
        .args(["select-layout", "-t", &target, layout])
        .status()
        .map_err(Error::Io)?;

    if !status.success() {
        return Err(Error::TmuxFailed("select-layout failed".into()));
    }

    Ok(())
}

/// Apply split only if the target window currently has a single pane
/// (i.e. it was just created). This avoids adding panes to an existing layout.
fn apply_split_if_single_pane(
    session: &str,
    window: Option<&str>,
    container: &str,
    shell: &str,
    split: &Split,
) -> Result<(), Error> {
    let target = match window {
        Some(name) => format!("{session}:{name}"),
        None => format!("{session}:0"),
    };

    let output = Command::new("tmux")
        .args(["list-panes", "-t", &target])
        .output()
        .map_err(Error::Io)?;

    let pane_count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .count();

    if pane_count == 1 {
        apply_split(session, window, container, shell, split)?;
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
