# dcon — Implementation Plan

## Goal

`dcon` solves a specific workflow problem: when working on a remote server via VS Code SSH Remote, you want **persistent, named terminal sessions that live inside the dev container** — not on the host. This gives you:

- Terminals that survive VS Code disconnects (tmux)
- A shell with the exact same environment as your dev container (docker exec)
- One session per project, multiple named windows per concern (api, worker, db, etc.)
- A sovereign, self-hosted equivalent to GitHub Codespaces terminal behaviour

## How it works (high level)

```
dcon [-n <window>]
  │
  ├─ 1. Find .devcontainer in cwd → abort if not found
  ├─ 2. Derive project codename from path
  ├─ 3. Find the running Docker container for this project
  │
  ├─ 4. tmux session "<codename>" exists?
  │        YES → attach (or switch to named window)
  │        NO  → create session with a shell inside the container
  │
  └─ 5. Named window (-n <name>)?
           YES → window "<name>" exists in session?
                   YES → select it
                   NO  → create it (exec into container), then select it
           NO  → use default window (window 0)
```

## Implementation steps

### Step 1 — CLI skeleton

- [ ] Set up `clap` with `derive` feature
- [ ] Accept optional `-n / --window <name>` argument
- [ ] Print parsed args (smoke test)

### Step 2 — `.devcontainer` detection

- [ ] Walk from `cwd` upward until `.devcontainer` is found or fs root is reached
- [ ] Return the project root path, or a clear error if not found

### Step 3 — Codename derivation

- [ ] Extract the folder name from the project root
- [ ] Check if the grandparent folder is named `workspace` or `workspaces`
- [ ] If yes, prepend the parent folder name with `_` separator
- [ ] Unit test all cases

### Step 4 — Docker container discovery

- [ ] Run `docker ps --format '{{.Names}}\t{{.Label "com.docker.compose.project"}}'` (or similar)
- [ ] Match the running container to the current project
  - Primary strategy: match on the devcontainer label (`devcontainer.local_folder`)
  - Fallback: match on container name heuristic
- [ ] Return container ID/name or a clear error if not running

### Step 5 — tmux session management

- [ ] Check if session `<codename>` exists (`tmux has-session`)
- [ ] If not: `tmux new-session -d -s <codename> -c <project-root> "docker exec -it <container> <shell>"`
- [ ] If `-n <window>`: check if window exists, create if not, then `tmux select-window`
- [ ] Finally: attach (`tmux attach-session`) or switch (`tmux switch-client`) depending on whether we are already inside tmux

### Step 6 — Shell detection

- [ ] Detect the default shell inside the container (`docker exec <container> sh -c 'echo $SHELL'`)
- [ ] Fall back to `/bin/bash` then `/bin/sh`

### Step 7 — Error handling & UX

- [ ] All errors printed to stderr with a clear message and non-zero exit code
- [ ] No `.devcontainer` → suggest running from a project directory
- [ ] Container not running → suggest opening the project in VS Code first
- [ ] `tmux` not found → suggest install
- [ ] `docker` not found → suggest install

### Step 8 — Integration tests

- [ ] Test codename derivation (pure logic, no subprocess)
- [ ] Test `.devcontainer` detection (with a temp dir)
- [ ] CLI smoke tests with `assert_cmd`

## Open questions

- **Container matching**: `devcontainer.local_folder` label is the most reliable signal. Need to verify it is set consistently across VS Code versions.
- **Nested workspaces**: what if `workspaces/scope/subscope/project`? Current plan takes only one level of scope. May revisit.
- **Windows Terminal / tmux inside tmux**: when already inside a tmux session, we switch rather than attach. Need to confirm behaviour for edge cases (nested tmux).
- **Shell**: should the shell be configurable (e.g. `dcon -n api --shell zsh`)? Deferred for now.
