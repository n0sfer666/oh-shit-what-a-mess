# OSWaM — oh-sht-what-a-mess

Консольная (TUI) утилита для анализа и **безопасной** очистки занятого места на macOS.
Находит мусор, который копит система (кэши, dev-артефакты, Docker, логи, корзина),
показывает физически освобождаемое место и помогает безопасно его удалить.

> Безопасность — инвариант №1. Утилита не кирпичит систему и не удаляет защищённое.
> Любое удаление — только по явному подтверждению.

## Установка

```sh
./install.sh            # собирает release и ставит `oswam` в ~/.cargo/bin
oswam                   # запуск
```

`./install.sh uninstall` — удалить. Альтернативно, без установки:

```sh
cargo build --release && ./target/release/oswam
```

Требуется Rust (`cargo`): https://rustup.rs. Каталог `~/.cargo/bin` должен быть в `PATH`.

## Использование

```sh
oswam                       # интерактивный TUI
oswam scan                  # вывод скана (человекочитаемо)
oswam scan --json           # машинный вывод для скриптов
oswam clean --safe --dry-run            # превью удаления безопасных категорий
oswam clean --category dev,browsers --trash --yes   # в Корзину
oswam clean --safe --delete --yes       # безвозвратно
```

Флаги `clean`: `--safe` (только Safe), `--category a,b`, `--dry-run`, `--trash` (по умолчанию),
`--delete` (безвозвратно), `--yes` (подтверждение для неинтерактивного запуска).

## Безопасность

- 4 уровня риска: **Safe / Caution / Danger / Never**.
- Встроенный неизменяемый `never`-список: SIP, root-owned, iCloud-стабы, `*.photoslibrary`.
- Пользовательский config (`~/.config/oswam/config.toml`): `protected_paths`, `ignore_globs`, `theme`.
- Docker — только `docker system prune`, файл `Docker.raw` не трогается.
- Физический (on-disk) расчёт размера — sparse/clones не обманывают.
- `--dry-run` + манифест удалённого. Корзина по умолчанию.
- Safe-by-default: предвыбраны только безопасные элементы.

## Навигация (TUI)

`j/k ↑↓` движение · `h/l` / `Tab` панели · `g/G` край · `Space` выбор · `o` группировка ·
`t` тема · `Ctrl+P` удалить · `?` справка · `q` выход.

## Что чистится

- **Системный мусор**: весь `~/Library/Caches/*` (по приложению — Steam, Cypress, Firefox,
  deno, next-swc и т.д.), `~/Library/Logs`, корзина.
- **Dev-окружение**: `~/.npm`, `~/.cache`, Xcode DerivedData/iOS DeviceSupport/Archives,
  CoreSimulator, Docker (`docker system prune`), недоступные симуляторы (`xcrun simctl delete unavailable`).
- **Большие данные (инфо)**: iOS-бэкапы (MobileSync) — только показ.
- **Снимки Time Machine** (только под sudo): локальные APFS-снимки. Удаляются все, **кроме
  последнего** (он сохраняется как свежий локальный recovery point).
- **Системные кэши `/Library/Caches`** (только под sudo, риск **Danger**): чистится только
  содержимое (папки остаются), SIP-защищённые пропускаются, всегда безвозвратно. Не предвыбираются
  — нужен явный выбор + подтверждение. `/System/Library/Caches` не трогается.

## Режим sudo

Без sudo чистится пользовательское пространство. Запуск `sudo oswam` добавляет локальные снимки
Time Machine и системные кэши `/Library/Caches`. Если запущено без прав, программа сообщит об этом
на стартовом экране и подскажет `sudo oswam`.

> **Риск системных кэшей**: после очистки возможны временные тормоза (приложения и macOS пересоздадут
> кэши), желателен перезапуск. Поэтому риск помечен как Danger, элементы не выбираются автоматически.
> Источник: [iBoysoft](https://iboysoft.com/wiki/library-caches-mac.html).

> Снимки Time Machine — это локальные почасовые копии (хранятся ~24ч, создаются системой
> не просто так). Они **не заменяют** внешний бэкап. Удаление старых снимков безопасно; **последний
> снимок не удаляется по умолчанию**. Источники: [Apple Support](https://support.apple.com/en-us/102154),
> [AppleInsider](https://appleinsider.com/articles/21/06/26/how-to-delete-time-machine-local-snapshots-in-macos).

## Архитектура

Cargo workspace: `oswam-core` (логика, без UI) · `oswam-tui` (ratatui) · `oswam-cli` (точка входа).

## Лицензия

MIT.
