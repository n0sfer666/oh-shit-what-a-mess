# OSWaM — oh-sh*t-what-a-mess

A terminal (TUI) utility to analyze and **safely** reclaim disk space on macOS.
It finds the junk the system accumulates — caches, dev artifacts, Docker, logs, trash —
shows the *physically* freeable space, and helps you delete it safely.

> Safety is invariant #1. OSWaM never bricks your system and never touches protected data.
> Every deletion happens only on explicit confirmation.

🇷🇺 [Русская версия](README.ru.md)

## Install

```sh
./install.sh            # builds release and installs `oswam` into ~/.cargo/bin
oswam                   # run
```

`./install.sh uninstall` removes it. Without installing:

```sh
cargo build --release && ./target/release/oswam
```

Requires Rust (`cargo`): https://rustup.rs. Make sure `~/.cargo/bin` is on your `PATH`.

## Usage

```sh
oswam                                               # interactive TUI
oswam scan                                          # print scan (human-readable)
oswam scan --json                                   # machine output for scripts
oswam clean --safe --dry-run                        # preview deleting safe categories
oswam clean --category dev,browsers --trash --yes   # move to Trash
oswam clean --safe --delete --yes                   # delete permanently
```

`clean` flags: `--safe` (Safe only), `--category a,b`, `--dry-run`, `--trash` (default),
`--delete` (permanent), `--yes` (confirm for non-interactive runs).

## Safety

- 4 risk levels: **Safe / Caution / Danger / Never**.
- Built-in immutable `never` list: SIP, root-owned, iCloud stubs, `*.photoslibrary`.
- User config (`~/.config/oswam/config.toml`): `protected_paths`, `ignore_globs`, `theme`.
- Docker: only `docker system prune` — the `Docker.raw` file is never touched.
- Physical (on-disk) size accounting — sparse files / clones can't fool it.
- `--dry-run` + a manifest of what was removed. Trash by default.
- Safe-by-default: only safe items are pre-selected.

## TUI navigation

`j/k ↑↓` move · `h/l` / `Tab` panels · `g/G` edge · `Space` select · `Enter` category → files /
toggle file · `a` recommended pick · `n` clear · `o` group · `t` theme · `Ctrl+P` delete ·
`?` help · `q` quit.

The sidebar shows `selected / available` per category and the bottom bar leads with the total that
will actually be removed. Files are listed biggest first. `Space` on a category picks only `✓ Safe`
and `▲ Caution` items that can go to the Trash; `✗ Danger` and permanent-only items are picked one
by one. After a Trash run, press `e` on the summary to permanently empty exactly what this run put
in the Trash — the rest of the Trash is left alone.

## What it cleans

- **System Data (macOS)** — what macOS Settings lumps into "System Data": all of
  `~/Library/Caches/*` (per app — Steam, Cypress, Firefox, deno, next-swc, …), `~/Library/Logs`,
  Saved Application State. Shown for reference only (never deleted): swap (`/System/Volumes/VM`),
  `sleepimage`, `/private/var/folders`, the Spotlight index and document versions (sudo), and
  **orphaned iOS Simulator runtimes** in `AssetsV2` that `simctl` no longer knows about — macOS
  never reclaims them and there is no supported way to remove them (only from Recovery with SIP off).
- **Trash**: `~/.Trash` contents (Danger — explicit choice only).
- **Dev environment**: `~/.npm`, `~/.cache`, Xcode DerivedData / iOS DeviceSupport / Archives,
  CoreSimulator, Docker (`docker system prune`), unavailable simulators (`xcrun simctl delete unavailable`).
- **Large data (info only)**: iOS backups (MobileSync) — display only.
- **Homebrew**: `brew cleanup` with `HOMEBREW_NO_AUTOREMOVE=1` (old versions and downloads only,
  dependency formulae are never auto-removed); Android SDK `.downloadIntermediates`;
  Steam `config/htmlcache` (skipped while Steam runs).
- **Time Machine snapshots** (sudo only): local APFS snapshots. All are removed **except the
  latest**, kept as a fresh local recovery point.
- **System caches `/Library/Caches`** (sudo only, **Danger**): contents only (folders kept),
  SIP-protected entries skipped, always permanent. Never pre-selected — requires explicit choice
  + confirmation. `/System/Library/Caches` is never touched.

## sudo mode

Without sudo, OSWaM cleans user space. Running `sudo oswam` adds Time Machine local snapshots
and fills the System Data group with `/Library/Caches`, `/Library/Logs`, crash reports, the unified
log (`log erase`) and Command Line Tools. Root cache entries are labelled with their full
`/Library/Caches/…` path. Note: under sudo, `clean --category system-data` also covers these
Danger/permanent targets — preview with `--dry-run` or add `--safe`. If launched without privileges, the welcome screen says
so and suggests `sudo oswam`.

> **System-cache risk**: after cleaning, expect temporary slowdowns (apps and macOS rebuild their
> caches); a restart is advisable. Hence the Danger level and no auto-selection.
> Source: [iBoysoft](https://iboysoft.com/wiki/library-caches-mac.html).

> Time Machine snapshots are local hourly copies (kept ~24h, created by the system for a reason).
> They are **not** a substitute for an external backup. Deleting old snapshots is safe; **the latest
> snapshot is kept by default**. Sources: [Apple Support](https://support.apple.com/en-us/102154),
> [AppleInsider](https://appleinsider.com/articles/21/06/26/how-to-delete-time-machine-local-snapshots-in-macos).

## Architecture

Cargo workspace: `oswam-core` (logic, no UI) · `oswam-tui` (ratatui) · `oswam-cli` (entry point).

## License

MIT.
