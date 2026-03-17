# dcon — Dev Container Connect

Seamlessly connects you to the dev container of a project via `tmux`. Keeps terminal sessions persistent and organized — one tmux session per project, shells running directly inside the Docker dev container.

## Install

### One-liner (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/totophe/remote-code-toolbox/main/scripts/install.sh | sh
```

Detects your OS and architecture, downloads the right binary from the latest release, and installs it to `~/.local/bin/dcon`.

To install to a different directory:

```sh
curl -fsSL https://raw.githubusercontent.com/totophe/remote-code-toolbox/main/scripts/install.sh | INSTALL_DIR=/usr/local/bin sh
```

### Via `cargo`

```sh
cargo install --git https://github.com/totophe/remote-code-toolbox dcon
```

### Manual

Download the binary for your platform from [GitHub Releases](https://github.com/totophe/remote-code-toolbox/releases/tag/latest):

| Platform | Binary |
|---|---|
| Linux x86_64 | `dcon-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `dcon-aarch64-unknown-linux-gnu` |
| macOS x86_64 | `dcon-x86_64-apple-darwin` |
| macOS Apple Silicon | `dcon-aarch64-apple-darwin` |

```sh
chmod +x dcon-*
mv dcon-* ~/.local/bin/dcon
```

## Requirements

- `tmux`
- `docker` (dev container must be running — VS Code starts it automatically)
- A `.devcontainer` folder at the project root

## Usage

```sh
# Connect to (or create) the default tmux window for this project
dcon

# Connect to (or create) a named window within the session
dcon -n api
dcon -n worker
dcon -n db

# Open 3 panes stacked top-to-bottom
dcon --stack 3

# Open 2 panes side-by-side in a named window
dcon -n api --side-by-side 2

# Override the shell (default: auto-detected from the container)
dcon --shell /bin/zsh
dcon -n api --shell /bin/zsh
```

Splits are only applied when the window is first created — reconnecting leaves the layout untouched.

## Configuration

Per-project shell can be set in `.devcontainer/dcon.json`:

```json
{
  "shell": "/bin/zsh"
}
```

Global fallback in `~/.config/dcon/config.json`:

```json
{
  "shell": "/bin/zsh"
}
```

Precedence: `--shell` flag > `.devcontainer/dcon.json` > `~/.config/dcon/config.json` > auto-detect.

## Codename derivation

The tmux session name is derived from the project path:

| `pwd` | Session name |
|---|---|
| `/home/totophe/workspaces/totophe/remote-code-toolbox` | `totophe_remote-code-toolbox` |
| `/home/totophe/workspaces/myproject` | `myproject` |
| `/home/totophe/projects/myapp` | `myapp` |

**Rule:** if the project lives two levels deep inside a `workspace` or `workspaces` folder, the scope (parent folder name) is prepended with `_`. Otherwise, just the folder name is used.
