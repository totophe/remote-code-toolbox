# remote-code-toolbox

A collection of CLI tools to improve the developer experience with remote VS Code — sovereign, self-hosted dev environments.

Written in Rust. Single static binaries, no runtime dependencies.

## Tools

| Tool | Description | Docs | Status |
|---|---|---|---|
| [`dcon`](docs/dcon/README.md) | Connect to a dev container's tmux session | [docs/dcon/](docs/dcon/) | In progress |

## Repo structure

```
remote-code-toolbox/
├── Cargo.toml              # workspace manifest
├── crates/
│   └── dcon/               # dcon binary crate
├── docs/
│   └── dcon/
│       ├── README.md       # usage & install
│       └── plan.md         # goal & implementation plan
└── .github/
    └── workflows/
        └── release.yml     # build & publish binaries on push to main
```

## Contributing

```sh
cargo build -p dcon
cargo test -p dcon
cargo build --workspace
```
