mod codename;
mod config;
mod devcontainer;
mod docker;
mod shell;
mod tmux;

use clap::Parser;

const MAX_PANES: u8 = 10;

#[derive(Parser, Debug)]
#[command(
    name = "dcon",
    about = "Connect to a dev container's tmux session from your project directory"
)]
struct Cli {
    /// Name of the tmux window to connect to (creates it if it doesn't exist)
    #[arg(short = 'n', long = "window", value_name = "NAME")]
    window: Option<String>,

    /// Shell to use inside the container (overrides config and auto-detection)
    #[arg(short = 's', long = "shell", value_name = "SHELL")]
    shell: Option<String>,

    /// Open N panes stacked top-to-bottom (only applied when creating the window)
    #[arg(long = "stack", value_name = "N", value_parser = parse_pane_count)]
    stack: Option<u8>,

    /// Open N panes side-by-side (only applied when creating the window)
    #[arg(long = "side-by-side", value_name = "N", value_parser = parse_pane_count)]
    side_by_side: Option<u8>,
}

fn parse_pane_count(s: &str) -> Result<u8, String> {
    let n: u8 = s.parse().map_err(|_| format!("'{s}' is not a valid number"))?;
    if n < 2 {
        return Err(format!("{n} is too few — minimum is 2"));
    }
    if n > MAX_PANES {
        return Err(format!("{n} exceeds the maximum of {MAX_PANES}"));
    }
    Ok(n)
}

fn main() {
    let cli = Cli::parse();

    if cli.stack.is_some() && cli.side_by_side.is_some() {
        eprintln!("error: --stack and --side-by-side are mutually exclusive");
        std::process::exit(1);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("error: cannot determine current directory: {e}");
        std::process::exit(1);
    });

    let project_root = devcontainer::find_project_root(&cwd).unwrap_or_else(|| {
        eprintln!("error: no .devcontainer folder found in {cwd:?} or any parent directory");
        eprintln!("hint: run dcon from inside a project that has a .devcontainer folder");
        std::process::exit(1);
    });

    let session = codename::derive(&project_root);

    let container = docker::find(&project_root).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    // Precedence: --shell flag > .devcontainer/dcon.json > ~/.config/dcon/config.json > auto-detect
    let cfg = config::Config::load(&project_root);
    let shell = cli
        .shell
        .or(cfg.shell)
        .unwrap_or_else(|| shell::detect(&container.name));

    let split = match (cli.stack, cli.side_by_side) {
        (Some(n), _) => Some(tmux::Split::Stack(n)),
        (_, Some(n)) => Some(tmux::Split::SideBySide(n)),
        _ => None,
    };

    let workspace_folder = devcontainer::workspace_folder(&project_root);

    tmux::connect(
        &session,
        cli.window.as_deref(),
        &container.name,
        &shell,
        &project_root,
        split,
        workspace_folder.as_deref(),
    )
    .unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
}
