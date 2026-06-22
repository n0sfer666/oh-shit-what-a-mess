# OSWaM — oh-sht-what-a-mess

Консольная (TUI) утилита для анализа и **безопасной** очистки занятого места на macOS.
Находит мусор, который копит система (кэши, dev-артефакты, Docker, логи, корзина),
показывает физически освобождаемое место и помогает безопасно его удалить.

> Безопасность — инвариант №1. Утилита не кирпичит систему и не удаляет защищённое.
> Любое удаление — только по явному подтверждению.

## Установка

```sh
cargo build --release
./target/release/oswam
```

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

## Архитектура

Cargo workspace: `oswam-core` (логика, без UI) · `oswam-tui` (ratatui) · `oswam-cli` (точка входа).

## Лицензия

MIT.
