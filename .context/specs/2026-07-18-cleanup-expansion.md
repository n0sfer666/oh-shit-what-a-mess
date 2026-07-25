# Спецификация: расширение методов очистки OSWaM

> Расширение реестра категорий на основе реального разбора «System Data» (159 GB).
> Дата: 2026-07-18. Статус: реализовано (registry + discover + never-list; T2 pass, 61 тест, скан 91.77 GB).
> Источник: скан машины + web-research (14 категорий).
> Базовая спека: `2026-06-22-oswam.md`. Инвариант №1 — безопасность.

## Контекст (реальная машина)

macOS · Apple Silicon · SIP on. Data-том: 285 GB занято / 160 GB свободно. «System Data» ≈ 159 GB.
Локальных снапшотов Time Machine — 0, sleepimage 2 GB → раздутость это живые файлы.

Карта крупнейших потребителей:

| Зона | Размер | Содержимое |
|------|--------|-----------|
| `~/Library/Application Support` | 36 GB | Steam 9.6, Claude 7.8, Google/Chrome 6.6, Arc 4.3 |
| `~/_dev` build-артефакты | 29 GB | Rust `target/` ~24, node_modules ~19 (перекрытие pnpm-hardlinks) |
| `Containers/Docker.raw` | ~21 GB | логич. 1 TB, sparse |
| `/Library/Developer` | 16 GB | CoreSimulator runtimes 13 + CommandLineTools 2.5 |
| `~/Library/Android/sdk` | 10 GB | system-images, эмулятор |
| `CoreSimulator/Devices` | 8 GB | iOS-симуляторы |
| `~/Library/Caches` | 3.5 GB | go-build 1.8, браузеры |
| `Group Containers` | 3.3 GB | Telegram media и др. |
| `pnpm · go · flutter` | 5.9 GB | dev store/SDK |

Согласованные параметры: агрессивность **до Danger**; охват **+ sudo**; `~/_dev` — **сканировать
и чистить build-артефакты (Caution)**; deep web-research безопасности — выполнен.

## Реестр — новые/уточнённые методы (приоритет по опасности удаления)

Порядок реализации = от наименее опасного. `P` = приоритет очистки (P1 = чистить первым).

### Safe (P1) — регенерируется автоматически, без потерь, можно предвыбирать
| Метод | Путь | Механизм | Почему безопасно |
|-------|------|----------|------------------|
| Native dev-кэши | `~/.npm`, `~/Library/pnpm`, `~/Library/Caches/{Yarn,go-build}`, pip | `npm cache clean --force`, `pnpm store prune`, `yarn cache clean`, `go clean -cache`, `pip cache purge` (НЕ rm) | Кэш-оптимизация, докачивается. **pnpm на хардлинках — только prune, иначе ломает node_modules всех проектов** |
| Xcode DerivedData | `~/Library/Developer/Xcode/DerivedData` | `DeletePath` (есть) | Только build/индексы, исходников нет, Xcode пересоздаёт |
| User Caches (contents) | `~/Library/Caches/*` | `DeleteContents` enumerate (есть) | Приложения перегенерируют |
| simctl unavailable devices | `CoreSimulator/Devices` осиротевшие | `xcrun simctl delete unavailable` (есть) | Девайсы без runtime и так не грузятся |

### Caution (P2) — вернётся, но пересборка/докачка/потеря индексов; НЕ предвыбирать
| Метод | Путь | Механизм | Почему Caution |
|-------|------|----------|----------------|
| ~/_dev build-артефакты | `~/_dev/**/{target,node_modules,dist,.next,build}` | Новая категория: скан по имени, `DeletePath`; физ-подсчёт с учётом клонов/хардлинков | Теряешь только время: `cargo build`/`npm install`. Не трогать при активной сборке |
| iOS Simulator runtimes | `/Library/Developer/CoreSimulator` (+Devices) | `xcrun simctl runtime delete` по версии; НЕ rm папки | Runtime 5-7 GB, docker из Xcode settings; не удалять активный |
| Android SDK/AVD/gradle | `~/Library/Android/sdk/system-images`, `~/.android/avd`, `~/.gradle/caches` | `sdkmanager --uninstall`, осиротевшие AVD, gradle `DeleteContents` | Образ 1.5-3 GB докачивается; не удалять активный AVD; остановить Android Studio |
| Docker reclaim | `Docker.raw` (не напрямую!) | `docker system prune` (есть) + `docker builder prune` + `docker/desktop-reclaim-space` | prune не сжимает .raw; reclaim-контейнер возвращает место; disk-size слайдер стирает всё |
| Xcode DeviceSupport/Archives | `~/Library/Developer/Xcode/{iOS DeviceSupport,Archives}` | `DeletePath` enumerate (есть, Caution) | DeviceSupport докачивается; **Archives — оставить не-загруженные релизы** |
| Браузер-кэши в App Support | `.../User Data/*/{Cache,Code Cache,GPUCache,Service Worker/CacheStorage}` | Точечный `DeletePath` только cache-подпапки; браузер закрыт | HTTP/GPU-кэш без потери паролей/истории/Spaces. НЕ трогать Cookies/Login Data/History/Local Storage |
| Системные логи (sudo) | `/private/var/log`, `/Library/Logs`, DiagnosticReports, CrashReporter | sudo, `DeleteContents` файлы (не папки), пропуск restricted/SIP | Отчёты, macOS пересоздаёт; удалять файлы не директории |

### Danger (P3) — потеря данных/состояния; только явное действие, отдельное предупреждение
| Метод | Путь | Механизм | Риск |
|-------|------|----------|------|
| Steam установленные игры | `.../Steam/steamapps/common/*` | Enumerate по играм, `DeletePath` по одной | Остаётся в аккаунте, но полная перекачка (десятки GB) |
| Крупные App Support данные | `~/Library/Application Support/{Claude,Google,…}` | InfoOnly + drill-down, вручную по под-папкам | Кэш+состояние+логины вместе; массово = разлогин/потеря истории |
| Медиа-кэш мессенджеров | `Group Containers/*.Telegram`, Teams/Slack | Только media-подпапка приложения; лучше нативная очистка в приложении | GC шарят данные — снос целиком ломает приложение; теряешь скачанные медиа |

### Never (P4) — жёстко в never-список
`/private/var/folders` · `/private/var/db` · `.Spotlight-V100` · `~/Library/Containers` (данные) ·
`/System`/SIP/root-owned · `~/Library/Keychains` · sleepimage/swapfiles (InfoOnly) ·
iCloud dataless-стабы · cloud-sync (`Yandex.Disk`) · TM local snapshots (0 шт; `sudo tmutil deletelocalsnapshots`, purgeable).

## Расчёт места (инвариант)
- **Физический on-disk** размер с учётом APFS-клонов и хардлинков (pnpm-store, Docker.raw sparse).
  Наивный `du` завышает (target+node_modules «43 GB», физически ≤29).
- **Purgeable space**: Finder и `df` расходятся — считать честно.
- Корзина не освобождает место до очистки — сообщать в итоге «освободится X».

## Оценка выигрыша
Safe + Caution ≈ **~90 GB** (dev-артефакты ~40, runtimes+devices ~21, Android ~10, Docker ~15-21,
кэши ~9) без Danger-данных. Danger (Steam/App ~24 GB) — сверх, по явному выбору.

## Порядок реализации (per-category, каждая — с апрувом и проверкой)
1. Native dev-кэши (Safe) — T2
2. `~/_dev` build-артефакты (Caution) — T2-T4
3. Simulator/Android runtimes (Caution) — T2
4. Docker reclaim + браузер-кэши (Caution) — T2-T3
5. sudo-логи (Caution) — T3
6. Danger-категория (Steam/App, InfoOnly+drill-down) — T3-T4
7. Дополнить never-список — T1

## Источники
CoreSimulator: deepclean.app, warchimede.hashnode.dev · Docker.raw: dissectmac.com · Android: dissectmac.com,
devsecopsnow.com · Xcode: swiftyplace.com · dev-кэши: devcleaner.app · браузер: sweepformac.com ·
логи: appleinsider.com · /var/folders: iboysoft.com · never: makeuseof.com · Steam: macpaw.com ·
snapshots: appleinsider.com · sleepimage: copyprogramming.com · Group Containers: cleanmymac.com.
