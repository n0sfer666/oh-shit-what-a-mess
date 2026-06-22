# Архитектура

Cargo workspace, ядро без UI.

## oswam-core (логика, ноль UI)
- `risk` — `RiskLevel` (Safe/Caution/Danger/Never), `PathFacts`, `classify` (never-сигналы,
  protected, process → Caution).
- `category` — декларативный реестр (`builtin_categories`), `CleanupKind`, `NativeSpec`.
- `paths` — раскрытие `~`, проверка ancestor.
- `fsops` — `FsOps` trait + `RealFs` (lstat/trash) + `FakeFs` (тесты); хелперы флагов.
- `size` — рекурсивный физический размер (устойчив к EPERM, без захода в симлинки).
- `facts` — сборка `PathFacts` из `Meta`.
- `config` — TOML (`protected_paths`, `ignore_globs`, `theme`), `is_protected`/`is_ignored`.
- `process` — `ProcessProbe` + `LsofProbe` (детект живых процессов через lsof).
- `manifest` — журнал удалённого (serde_json).
- `delete` — `delete_target` (dry-run, Trash/Permanent, без симлинков, устойчив к ошибкам).
- `docker` — `estimate` (system df) / `run_clean` (system prune), парсер размеров.
- `scan` — оркестрация → `ScanResult` (serde).
- `select` — фильтр выбираемого (deletable, safe-only, категории).
- `format` — human_bytes.

## oswam-tui (ratatui)
- `app` — состояние, навигация, грузппировка, предвыбор Safe.
- `input` — обработка `Key` (vim+стандарт).
- `theme` — палитра dark/light, цвет+символ риска.
- `detect` — авто-детект темы (COLORFGBG).
- `event` — маппинг crossterm → `Key`.
- `render`/`panels` — layout (сайдбар/описание/таблица/action) + help-оверлей; snapshot-тесты.
- `run` — terminal event loop, возвращает выбранное на proceed.

## oswam-cli
- `cli` — clap (сабкоманды scan/clean, default → TUI).
- `context` — сборка окружения + `run_scan` на реальной ФС.
- `perform` — исполнение удаления (fs + native) → manifest.
- `output` — печать скана/итога.
- `commands` — хендлеры scan/clean/tui.

## Поток данных
registry → expand(~) → meta(FsOps) → facts → classify → physical_size → ScanResult →
(TUI выбор | CLI select) → delete_target/docker → manifest → summary.
