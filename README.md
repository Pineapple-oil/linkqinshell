# LinkQinShell

LinkQinShell is a Windows-first, cross-platform server management desktop app built with Rust and Slint.

This repository currently contains the P0 MVP foundation:

- Rust workspace with P0 crates.
- Slint desktop shell.
- Core connection/session/transfer/host-key models.
- Master-password encryption and sensitive log redaction.
- SQLite schema and connection repository.
- SSH password/private-key authentication backend using `ssh2`.
- SFTP list/upload/download/rename/mkdir/delete backend.
- Terminal buffer/search abstraction.
- Roadmap tracking P0/P1/P2/P3 features from the development document.

## Requirements

Install Rust before building:

```powershell
winget install Rustlang.Rustup
rustup default stable
```

## Commands

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p linkqinshell-app
```

## Local Data

Runtime data is stored under the platform-local `LinkQinShell` data directory.
On Windows this resolves to `%LOCALAPPDATA%\LinkQinShell`.

- Database: `linkqinshell.db`
- Log file: `app.log`

## Roadmap

See `docs/feature-roadmap.md` for the full P0/P1/P2/P3 status map.
