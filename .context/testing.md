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
  facts, config (protected/ignore), process ancestry и имена процессов (`running`),
  `scan/tests/blocked.rs` (блокировка запущенным приложением, companion-манифест, скрытые
  файлы при перечислении), manifest, delete (dry-run/trash/permanent,
  симлинки; `tests/trashed.rs` — дочистка Корзины трогает только своё по ino/dev, в т.ч. при доме-алиасе), scan, select, docker-парсер. Деструктив — только `FakeFs`.
- **Инварианты реестра** (`registry/tests/`): `catalog` — состав целей; `claude` — цели
  Claude Desktop (все `Caution` + `blocking_app`, образ с манифестом, сессии не цель);
  `nesting` — вложенность с учётом `discover` (корень покрывает только именованные каталоги);
  `natives` — argv docker/simctl-команд; `invariants` — VM-образы (поимённое исключение
  `rootfs.img`: только с `blocking_app` и не мягче `Caution`), аргументы натив-команд, привязка к пользователю,
  `privileged == needs_root` (пользовательская команда не смеет остаться root'ом под sudo);
  `catalog::irreplaceable_user_data_is_never_softer_than_danger` (данные пользователя в
  `big-data` не бывают мягче `Danger`); `honesty` — риск и обещание размера (кэш, стоящий перекачки, не может быть `Safe`, потому что
  TUI автовыбирает все `Safe`; измеряемая натив-цель обязана иметь `probe`, реальный путь и
  стоять в аудированном списке — доказано, что её команда чистит именно этот путь).
- **oswam-tui**: unit на app/input/theme/detect/event/describe/explain/sidebar; интеграционные
  `tests/behavior.rs` + `tests/scrolling.rs` + `tests/overview.rs` (выбрано/доступно, Space на
  категории без Danger/«только безвозвратно», симметричное снятие, a/n, Enter, сортировка) +
  `tests/purge.rs` (`e` на итоге, честный итог дочистки: «Остались в Корзине», ошибки) (общие фикстуры — `tests/support/`);
  snapshot (`TestBackend` + `insta`) на layout, help-оверлей и модалку подтверждения.
- **oswam-cli**: парсинг clap, disposition, текст итога (`summary_lines`: превью не смеет
  отчитываться в прошедшем времени о том, чего не делал).
- **Условие коммита**: `cargo test` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`.
- Снапшоты обновляются вручную: `INSTA_UPDATE=new cargo test -p oswam-tui --test snapshots`,
  затем `mv *.snap.new *.snap` (бинаря `cargo-insta` в окружении нет).
- Текущее покрытие: 371 тест.

## Ручная проверка: Claude Desktop (только живая машина)
1. Claude запущен → `cargo run -q -p oswam-cli -- scan --json`: у категории `claude`
   `total_bytes` = 0, у всех записей `"blocking_app": "Claude"`, `"blocked": true`.
   `oswam scan` — у записей суффикс «⏸ закройте Claude», внизу строка
   «Заблокировано: Claude запущен … (целей: N)». В TUI: в сводке категории строка «⏸ …»,
   в сайдбаре «⏸ <размер>», описание записи «Заблокировано…», `Space` её не выбирает.
2. Закрыть Claude (⌘Q, без процесса `Claude` в `pgrep -x Claude`) → перезапустить oswam:
   записи выбираемы, у `rootfs.img` в `companions` есть `.rootfs.img.origin`.
   Выбрать записи, но ДО подтверждения снова открыть Claude → подтвердить: в итоге
   «Пропущено: N — приложение снова запущено…», файлы на месте.
3. Удалить `rootfs.img` в Корзину → открыть Claude → запустить Cowork: образ скачивается
   заново (~10 GB), `sessiondata.img` на месте, прошлые сессии Cowork открываются.
   `.cowork-adopted` при этом остаётся — проверить, что Claude не считает бандл готовым
   (ошибка старта VM). Если Claude НЕ пересоздал образ — откатить из Корзины и понизить
   цель обратно до `InfoOnly` (либо добавить `.cowork-adopted` в companions — решает пользователь).

## Ручная проверка: System Data (только живая машина, ничего не удаляет)
1. `cargo run -q --release -p oswam-cli -- scan --json` → категория `system-data`
   («System Data (macOS)») первая; в ней `~/Library/Caches/*`, `~/Library/Logs`, Saved State и
   `info_only`/`never`: `/System/Volumes/VM`, `/private/var/vm`, `/private/var/folders` с
   ненулевым размером. `total_bytes` категории не включает `info_only`.
2. Есть рантайм, которого нет в `xcrun simctl runtime list -v` (сравнить с
   `ls /System/Library/AssetsV2/com_apple_MobileAsset_iOSSimulatorRuntime`) → запись
   «iOS X.Y (BUILD) — осиротевший рантайм», `info_only`/`never`, размер ≈ `du -sh` ассета.
   Все рантаймы зарегистрированы → такой записи нет. Без Xcode/CLT → записи нет, скан не падает.
3. Steam запущен → у `Steam/config/htmlcache` в `big-data` `"blocked": true`.
4. `oswam scan --json`: у `dev`-записи Homebrew `native.clean` =
   `["env","HOMEBREW_NO_AUTOREMOVE=1","brew","cleanup"]`.
5. `sudo oswam scan --json` → в `system-data` появляются `/Library/Caches/*` (Danger, display —
   полный путь `/Library/Caches/<имя>`),
   `/Library/Logs`, CrashReporter, `log erase`, Spotlight/DocumentRevisions (`info_only`).
6. TUI: описание сироты упоминает simctl и SIP; VM/var/folders — «управляет сама система»;
   без sudo в сводке System Data строка «…видна только под `sudo oswam`», под sudo её нет;
   Saved Application State — текст про окна, не «кэш».

## Backlog
- Интеграционные тесты CLI на tmpdir-фикстурах (реальный RealFs на временных путях).
- Полный обход ФС (поиск крупных неизвестных файлов) — не реализован (опционально по спеке).
- In-TUI модалка Корзина/Безвозвратно (сейчас prompt на stdout после выхода из TUI).
