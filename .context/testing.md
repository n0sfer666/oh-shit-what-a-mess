# Тестовая стратегия

- **Уровень**: T2-T4 (деструктив + критичный путь). Каждая фича — с падающего теста.
- **oswam-core**: unit на классификатор риска (в т.ч. «занято процессом» не поднимает
  `Caution` до `Danger`), дом вызывающего под sudo (`sudo::caller_from`/`resolved_home` —
  чистые функции с подставным окружением, без зависимости от машины),
  разбор `scan/tests/` на `basics`/`discovered`/`grouped`/`native`/`risk` (`discovered` пинит
  дедуп найденных каталогов: синонимичные и вложенные корни считаются один раз, корень-симлинк
  на другой том всё равно сканируется и помечается `permanent_only`, а защита переживает
  канонизацию в обе стороны — и по реальному пути, и по алиасному из конфига),
  физ. размер (incl. устойчивость к EPERM),
  facts, config (protected/ignore), process ancestry, manifest, delete (dry-run/trash/permanent,
  симлинки), scan, select, docker-парсер. Деструктив — только `FakeFs`.
- **Инварианты реестра** (`registry/tests/`): `catalog` — состав целей; `natives` — argv
  docker/simctl-команд; `invariants` — вложенность
  (включая измеряемые нативы), VM-образы, аргументы натив-команд, привязка к пользователю,
  `privileged == needs_root` (пользовательская команда не смеет остаться root'ом под sudo);
  `catalog::irreplaceable_user_data_is_never_softer_than_danger` (данные пользователя в
  `big-data` не бывают мягче `Danger`); `honesty` — риск и обещание размера (кэш, стоящий перекачки, не может быть `Safe`, потому что
  TUI автовыбирает все `Safe`; измеряемая натив-цель обязана иметь `probe`, реальный путь и
  стоять в аудированном списке — доказано, что её команда чистит именно этот путь).
- **oswam-tui**: unit на app/input/theme/detect/event/describe/explain; интеграционные
  `tests/behavior.rs` + `tests/scrolling.rs` (общие фикстуры — `tests/support/`);
  snapshot (`TestBackend` + `insta`) на layout, help-оверлей и модалку подтверждения.
- **oswam-cli**: парсинг clap, disposition, текст итога (`summary_lines`: превью не смеет
  отчитываться в прошедшем времени о том, чего не делал).
- **Условие коммита**: `cargo test` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`.
- Снапшоты обновляются вручную: `INSTA_UPDATE=new cargo test -p oswam-tui --test snapshots`,
  затем `mv *.snap.new *.snap` (бинаря `cargo-insta` в окружении нет).
- Текущее покрытие: 262 теста (core 200, tui 55, cli 7).

## Backlog
- Интеграционные тесты CLI на tmpdir-фикстурах (реальный RealFs на временных путях).
- Полный обход ФС (поиск крупных неизвестных файлов) — не реализован (опционально по спеке).
- In-TUI модалка Корзина/Безвозвратно (сейчас prompt на stdout после выхода из TUI).
