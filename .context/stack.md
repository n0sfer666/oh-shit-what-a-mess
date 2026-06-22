# Стек

- **Язык**: Rust (edition 2021, rust-version 1.80), Cargo workspace, resolver 2.
- **Домен**: systems / native CLI+TUI, macOS (Apple Silicon + Intel).
- **Крейты**:
  - core: `serde`, `serde_json`, `toml`, `thiserror`, `glob`, `dirs`, `chrono`, `libc` (unix); dev: `tempfile`.
  - tui: `ratatui` 0.29, `crossterm` 0.28; dev: `insta`.
  - cli: `clap` 4 (derive), `anyhow`, `serde_json`, `dirs`.
- **Бинарь**: `oswam` (из `oswam-cli`).
- **Платформенное**: `libc::lstat` для `st_blocks`/`st_uid`/`st_flags` (dataless `SF_DATALESS`,
  SIP `SF_RESTRICTED`). Trash — перенос в `~/.Trash`.
