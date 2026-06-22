# Тестовая стратегия

- **Уровень**: T2-T4 (деструктив + критичный путь). Каждая фича — с падающего теста.
- **oswam-core**: unit на классификатор риска, физ. размер (incl. устойчивость к EPERM),
  facts, config (protected/ignore), process ancestry, manifest, delete (dry-run/trash/permanent,
  симлинки), scan, select, docker-парсер. Деструктив — только `FakeFs`.
- **oswam-tui**: unit на app/input/theme/detect/event; snapshot (`TestBackend` + `insta`) на
  layout и help-оверлей.
- **oswam-cli**: парсинг clap, disposition.
- **Условие коммита**: `cargo test` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`.
- Текущее покрытие: 70 тестов (core 45, tui 21, cli 5 — без учёта doc/snapshot binaries).

## Backlog
- Интеграционные тесты CLI на tmpdir-фикстурах (реальный RealFs на временных путях).
- Полный обход ФС (поиск крупных неизвестных файлов) — не реализован (опционально по спеке).
- In-TUI модалка Корзина/Безвозвратно (сейчас prompt на stdout после выхода из TUI).
