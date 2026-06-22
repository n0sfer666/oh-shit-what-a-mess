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

Часть «System Data» требует sudo и в v1 не трогается — `oswam scan` подсказывает, что ещё можно
освободить вручную (локальные снимки Time Machine `sudo tmutil thinlocalsnapshots /`, `/Library/Caches`).

## Архитектура

Cargo workspace: `oswam-core` (логика, без UI) · `oswam-tui` (ratatui) · `oswam-cli` (точка входа).

## Лицензия

MIT.
