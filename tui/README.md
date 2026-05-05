# rulof-tui

SSH server (russh) that serves a terminal UI (ratatui) mirroring the site content.

## Run

```bash
cd tui
cargo run
```

- Listens on **port 22** by default. Set `environment=development` in `.env` (or the environment) to use **2222** instead.
- Host key: `~/.config/rulof-tui/host_key` (created on first run).

## Build

```bash
cargo build --release
```

Binary: `target/release/rulof-tui`.
