# Архитектура

Cargo workspace, ядро без UI.

## oswam-core (логика, ноль UI)
- `risk` — `RiskLevel` (Safe/Caution/Danger/Never), `PathFacts`, `classify` (never-сигналы,
  protected). Живой процесс поднимает риск ровно до `Caution` и не выше: «занято» — это
  предупреждение о занятости, а не заявление о данных пользователя, а `Danger` по шкале
  означает именно необратимые данные (иначе `target/`, который держит rust-analyzer,
  подписывался бы как «данные пользователя»).
- `category` — типы реестра (`CleanupKind`, `NativeSpec`, `Target`) + `builtin_categories`.
- `registry/` — сам каталог целей по категориям (`system_data`, `dev`, `simulators`, `containers`,
  `browsers`, `claude`, `projects`, `bigdata`, `trash`) + `registry/tests/` (каталог и инварианты).
  `system_data` (`SYSTEM_DATA_ID = "system-data"`) — всё, что macOS относит к «Системным данным»:
  generic-кэши/логи дома, sudo-цели (единственная категория с `needs_root`) и InfoOnly-справка
  (VM-том, `sleepimage`, `/private/var/folders`, Spotlight, DocumentRevisions).
- `runtimes/` — осиротевшие рантаймы iOS-симулятора в `AssetsV2`: `registered_images` парсит
  `xcrun simctl runtime list -v` (без заголовка `== Disk Images ==` → `None`, т.е. «сирот нет»),
  число `Image Path:` обязано совпасть с `Total Disk Images: N`, иначе тоже `None`),
  `orphan_assets` — `*.asset`, не упомянутые simctl, `orphan_entry` — `InfoOnly`/`Never`
  (ignored/protected сирота скрыта), `find(fs, guard, run, read)` — оркестрация с внедряемыми
  запуском команд и чтением Info.plist (тесты на FakeFs). Нативного удаления нет (simctl их не
  видит), поэтому только показ.
- `sysdata/` — `root_caches` (дети `/Library/Caches` под sudo: `Danger`, безвозвратно, подпись —
  полный путь, отклонённые guard'ом → `InfoOnly`/`Never`) и `absorb(result, entries)` — влить
  записи в `system-data` (создаёт категорию, если её нет), к итогам прибавляется только
  `is_deletable`.
- `native::run_within(argv, privileged, limit)` — запуск с дедлайном (kill по таймауту); им
  ходят пробы, которые идут на каждом скане (`xcode-select`, `simctl runtime list`), чтобы
  зависший CoreSimulatorService не вешал скан.
- `/private/var/folders` (InfoOnly) меряется целиком: ~3 с на живой машине из ~30 с скана,
  без root — `partial`. Стоимость принята осознанно: это заметная часть System Data.
  `claude` — Claude Desktop: Chromium-кэши (общий `browsers::CACHE_DIRS`, depth 0 — только верхний
  уровень, внуки вроде `claude-code/*/Cache` не трогаются), встроенные сборки
  `claude-code/`/`claude-code-vm/` поштучно (`versioned` → метка `claude-code/<ver>`) и
  базовый образ Cowork-VM `rootfs.img` вместе с манифестом `.rootfs.img.origin`.
  `sessiondata.img` (сессии Cowork) целью не бывает. Решение — ADR `2026-10-06-claude-desktop`.
- `paths` — раскрытие `~`, проверка ancestor.
- `refetch` — список имён кэшей, которые стоят перекачки (`homebrew`, `go-build`, `pip`,
  `cocoapods`, …). `costs_a_download(path)` поднимает риск такого потомка до `Caution` даже
  под `Safe`-родителем: `~/Library/Caches` перечисляется целиком, но Homebrew-кэш не может
  автовыбираться как Safe вместе с кэшем Safari. Список ведётся полным (43 имени: `cypress`,
  `sccache`, `rustup`, `jetbrains`, `poetry`, …) — TUI автоматически отмечает всё `Safe`,
  поэтому неопознанный многогигабайтный кэш уехал бы в удаление одним Enter.
- `sudo` — кто стоит за root'ом (`SUDO_UID`/`SUDO_GID`/`SUDO_USER`) + его настоящий дом
  (`getpwnam`/`getpwuid`, а не `$HOME`). `user_home()` — единственный источник дома для
  `RealFs::new`, `default_config_path` и `Env`: под `sudo -H` (или sudoers без
  `env_keep += HOME`) `$HOME` равен `/var/root`, и без этого все `~/…` цели молча
  раскрывались бы в дом рута, конфиг пользователя (`protected_paths`) переставал бы
  действовать, а `can_trash` сравнивал бы с `/var/root/.Trash` — то есть всё удалялось бы
  безвозвратно вместо Корзины. `drop_privileges(cmd)` роняет привилегии дочернего процесса
  обратно на пользователя (`uid`/`gid` + `USER`/`LOGNAME`/`HOME`). Собственного `pre_exec` тут
  быть не может: std запускает pre_exec-замыкания ПОСЛЕ своих `setgroups`/`setgid`/`setuid`,
  поэтому `setgroups` из замыкания вызывается уже не от root и валится `EPERM` — а это ошибка
  spawn, то есть провалившаяся проба и молча исчезнувшая из скана цель. Список доп. групп
  сбрасывает сам std при уходе с root'а. `NativeSpec.privileged`
  (ставится `Target::needing_root`) — единственный признак, что команду и правда надо
  выполнять от root: `npm cache clean`, `pnpm store prune`, `xcrun simctl delete` под
  `sudo oswam` иначе пересоздали бы пользовательские кэши с владельцем root:wheel и
  чистили бы CoreSimulator самого root'а. Пробы и estimate идут тем же путём, поэтому
  `docker system df` под sudo показывает docker пользователя, а не root'а.
- `fsops/` — `FsOps` trait (+ `can_trash`, `canonicalize`) + `real::RealFs` (lstat/trash,
  `ensure_trash` создаёт `~/.Trash` с владельцем вызывающего, а не root'а) + `fake::FakeFs` (тесты);
  `Meta` с `nlink`/`ino`/`dev`, хелперы флагов, `is_inside_trash`. `RealFs` держит `$HOME` в обеих
  формах (сырой и realpath) и признаёт «своим» путь по любой из них, иначе символическая ссылка
  в `$HOME` превратила бы весь скан в `permanent_only`. `can_trash` дополнительно требует того же
  тома (`Meta.dev`): переезд в Корзину через границу тома всё равно упадёт с EXDEV, честнее сказать
  об этом заранее.
- `size` — `measure`/`measure_where` → `Measure { bytes, shared_bytes, partial, kept }`: рекурсивный физический
  размер (устойчив к EPERM — тогда `partial = true`, без захода в симлинки). Жёсткая ссылка
  (`nlink > 1`) обещает только свою долю блоков (`physical_bytes / nlink`): удаление одной
  ссылки из N не освобождает файл, а все N ссылок внутри удаляемого множества в сумме дают
  ровно полный размер. APFS-клоны не схлопываются — у клона свой инод, и `stat` показывает
  полный размер до расхождения блоков. `measure_where` не прибавляет собственные блоки
  каталога, если хотя бы один потомок исключён (каталог переживёт удаление). `shared_bytes` —
  сколько из `bytes` лежит в файлах с `nlink > 1`: UI отдельно предупреждает, что эта часть
  освободится только вместе с последней ссылкой.
- `guard` — `Guard { fs, config, home, allow_root_owned }` (билдер `Guard::new(…)
  .allowing_root_owned(bool)`), `allows(path)`: единая проверка ignore/protected/never перед
  удалением и при сборе companions. `admits(path)` — только ignore/protected: им меряются
  `InfoOnly`-цели (иначе root-owned/SIP-справка под `/System` весила бы 0). Root-owned разрешается точечно — только для цели с
  `needs_root` под sudo, а не глобально на весь запуск.
- `grouping` — склейка `X.avd` + `X.ini` в одну цель (primary + companions).
- `discover/` — поиск build-артефактов по имени/префиксу с маркерами генерации. Корень-симлинк
  обходится как обычный каталог: `~/dev → /Volumes/Work/dev` — это единственный способ увидеть
  проект, и выбрасывать такой корень значило бы молча спрятать гигабайты. `scan` обходит корень
  по realpath (алиас остаётся только в `display`), поэтому `ScanEntry.path` всегда реальный —
  лексические проверки (`is_protected`, `is_ignored`, `is_never_path`, lsof) не промахиваются.
  Двойной учёт снимается дедупом НАЙДЕННЫХ каталогов (`seen_dirs` на весь скан), а не корней:
  это разом закрывает и синонимичные корни (`~/dev → ~/Developer`), и вложенные
  (`~/Projects → ~/Developer/Projects`). Симлинк-потомок внутри обхода по-прежнему пропускается. Неоднозначные имена (`target`) требуют доказательства —
  маркер внутри (`CACHEDIR.TAG`, `.rustc_info.json`, CMake-файлы) либо манифест рядом
  (`Cargo.toml`, `pom.xml`, `build.sbt`, `build.gradle[.kts]`): каталог меток `target/` в
  ML-проекте артефактом не является. Точное `build` строже: манифест рядом за него не ручается
  (Rust собирает в `target/`, а `build/` в том же репозитории — обычно исходники), поэтому годится
  только маркер внутри самого каталога.
- `facts` — сборка `PathFacts` из `Meta`.
- `config/` — TOML (`protected_paths`, `ignore_globs`, `theme`), `is_protected`/`is_ignored`.
  `resolved(fs, home)` (вызывается один раз при загрузке) держит каждую запись в трёх формах —
  как написано, с раскрытым `~` и с realpath: пути в скане канонические не все (симлинк-таргет
  намеренно остаётся алиасным), а пользователь пишет в конфиг то, что видит в UI. Совпадение по
  любой форме считается совпадением, поэтому набор правил только расширяется. Раскрытая форма
  обязательна отдельно от realpath: `is_ignored` сравнивает глоб с абсолютным путём и `~` сам
  не раскрывает, а realpath-формы может не быть вовсе (несуществующий или относительный префикс —
  последний не резолвится сознательно, иначе смысл конфига зависел бы от CWD запуска).
  Глоб резолвится по литеральному префиксу до первого `*`/`?`/`[`, и производные формы этого
  префикса экранируются (`Pattern::escape`): том `/Volumes/My [Backup]` иначе стал бы классом
  символов и правило молча перестало бы совпадать.
- `process` — `ProcessProbe` + `LsofProbe`: один вызов `lsof -Fcn -w +c 0` даёт и открытые
  пути (`holds` → риск не ниже `Caution`), и имена процессов (`running(app)`, точное
  совпадение: `Claude` ≠ `Claude Helper` ≠ `claude`). Если `lsof` ничего не вернул, `running`
  = true (fail-closed): лучше ложная блокировка, чем удаление из-под работающего приложения.
  `Target.blocking_app` → `ScanEntry.blocking_app` (всегда) + `ScanEntry.blocked` (состояние
  при скане; ставится только у удаляемой записи — `InfoOnly`/`Never`, в том числе ставшая
  `InfoOnly` из-за защищённого companion, не обещает «закройте и удалите»): заблокированная
  запись видна с размером, но `is_deletable` = false, в
  «освобождаемое» не входит, в TUI идёт отдельной строкой «⏸» (не в «ⓘ Только просмотр»; в таблице метка « ⏸ »),
  в CLI — суффикс «⏸ закройте <app>» и итоговая строка.
- `blocking` — `Recheck::before_delete`: перед удалением свежий `lsof` (только если среди
  выбранного есть `blocking_app`); приложение открыли после скана ИЛИ путь/companion держит процесс (осиротевший helper)
  → запись пропускается
  (TUI: `Summary.blocked`, CLI: «пропущено …»). Оба раннера (`runners.rs`, `perform.rs`).
  `blocking_app` бывает только у путевых целей (инвариант: не у `NativeCommand`).
- `manifest` — журнал удалённого (serde_json).
- `delete/` — `Deleter { disposition, dry_run }` → `remove(Guard, …) -> DeleteReport
  { done, failed }`: primary удаляется первым (при неуспехе companions не трогаются), дети
  проходят через `Guard` (защищённый потомок остаётся, вместе с ним остаётся и родительский
  каталог), `effective` переводит в `Permanent` то, что нельзя положить в Корзину (вне `$HOME`,
  внутри `.Trash`, на другом томе). Ошибка чтения каталога возвращается наверх; частичные отказы попадают в
  `failed`, а не проглатываются. Байты в отчёте считаются только по тому, что реально ушло с
  диска: `walk::plan_all` снимает метаданные ВСЕГО набора (primary + companions) ДО первой
  мутации — `Plan { Leaf / Dir / Blocked }`, — и трэш/удаление/dry-run ходят уже по плану.
  Иначе `nlink` уезжает по ходу удаления: вторая жёсткая ссылка после удаления первой
  выглядит как единственная и обещание удвоилось бы. Байты засчитываются ровно на успешной
  операции. Элемент, где ничего не удалилось без единой ошибки, попадает в `done` с нулём и
  `untouched: true`; `partial` — это «часть ушла, часть осталась», флаги взаимоисключающие.
  `primary_items` возвращает `Primaries { items, refused }`: цель, целиком отклонённая guard'ом
  (`items` пуст, `refused`), тоже отчитывается как `untouched`, а не исчезает пустым отчётом.
  Companions спасает только незавершённое удаление самого primary; отфильтрованный guard'ом
  сосед (чужой `*.lock` рядом) их не спасает — иначе `X.ini` осиротеет при удалённом `X.avd`.
  Зато защищённый **companion** отменяет всю единицу: комплект удаляется целиком или никак,
  отчёт — `untouched`. Перенос в Корзину пишет в отчёт `trashed: Vec<Trashed { path, bytes, ino,
  dev }>` — куда уехал элемент и его идентичность (ino/dev снимаются ДО `rename`, `move_to_trash`
  возвращает путь назначения). `purge_trashed(guard, items)` удаляет безвозвратно только прямого
  потомка `~/.Trash` с совпавшим ino/dev: чужой файл под тем же именем, восстановленный
  пользователем элемент, сам корень Корзины и всё вне её — `untouched`. Родитель сверяется с Корзиной
  через `canonicalize` (дом-симлинк). Нет ino/dev до переноса — перенос не делается (`failed`).
  Под root дочистка не предлагается и не запускается (TOCTOU в пишущем пользователем `~/.Trash`). Риск записи не влияет на способ удаления: `Danger` не форсирует
  `Permanent`, Корзина восстановима и остаётся дефолтом; в `Permanent` переводит только
  невозможность трэша (`effective`) или явный выбор пользователя.
- `docker` — `estimate` (кэш вывода на процесс, сбрасывается после native-очистки),
  `run_clean`, парсер размеров и фильтр по типу ресурса. Строка режется по табу или по первому
  двоеточию, размер берётся из хвостовой скобки и только затем из первого токена: так
  `Total Disk Images: 5 (39.4G)` (`simctl runtime list`) читается как 39.4 GB, а не как 5 байт,
  и при этом `1.5GB (50%)` (`docker system df -v`) остаётся 1.5 GB. У `NativeSpec` два разных канала:
  `probe: Vec<Vec<String>>` — цепочка проверок инструмента, каждая должна выйти с нулём (нет
  бинаря ИЛИ ненулевой выход → цель скрыта из скана), stdout проб не читается как размер;
  `estimate` — единственный источник размера (вывод парсится как байты). Цепочка нужна, чтобы
  `xcrun --find simctl` шёл только после `xcode-select -p`: иначе `xcrun` на машине без CLT
  поднимает GUI-установщик прямо во время скана. Пустой `estimate` → размер честно неизвестен
  (`size_unknown`), провалившийся `estimate` → цель скрыта, а не показана нулём.
- `scan/` — `mod` (оркестрация, `ScanCtx::guard()`/`guard_for(target)`, `ScanEntry::guard(…)`,
  `ScanEntry::size_label()`, `reclaimable_bytes`) + `target` (диспетчер одной цели:
  native/discover/enumerate/grouped) + `entry` (построение одного `ScanEntry`: ignore → meta →
  facts → classify → measure) → `ScanResult` (serde). `ScanEntry` несёт сигналы риска для UI:
  `partial` (часть содержимого недоступна — «не менее X»), `shared_bytes` (доля в общих файлах),
  `size_unknown` (размер заранее неизвестен — не путать с честным нулём; `size_label()` печатает
  «—», а не «0 B»), `needs_root`, `permanent_only` (Корзина недоступна).
  «Измеряемая нативная» цель (`Target.measured`, конструктор `registry::native_at`) — путь
  реально существует и измеряется как обычная цель, но чистится нативной командой
  (`npm cache clean --force`): размер честный, а удаление делает штатная утилита. Такая цель
  скрывается, если её инструмент не прошёл `probe`; если инструмент есть, а путь уехал
  (свой `npm config get cache`), цель остаётся с честным `size_unknown`, а не исчезает.
  Список измеряемых нативов аудируемый (`registry/tests/honesty.rs`): команда обязана чистить
  ровно измеренный путь — `yarn cache clean` этому не удовлетворял (Classic чистит
  `~/Library/Caches/Yarn`, Berry — кэш проекта и зависит от cwd), поэтому
  `~/.yarn/berry/cache` — обычная цель `DeleteContents`.
  Группировка (`X.avd` + `X.ini`): если хоть один companion отклонён guard'ом, вся единица
  становится `InfoOnly` — показывается с размером, но не удаляется, чтобы не осиротить пару.
  Те же правила (`scan/companions.rs`) действуют для одиночной цели с `Target.companions` —
  имена соседей рядом с путём цели (`rootfs.img` + `.rootfs.img.origin`); отсутствующий сосед
  просто не обещается.
  Перечисление `versioned` (каталоги версий: `claude-code/`) пропускает скрытые ФАЙЛЫ
  (`.sdk-version`) — это метаданные каталога, а не версия; скрытые каталоги остаются.
  Обычное `enumerated` скрытые файлы показывает.
- `select` — фильтр выбираемого (deletable, safe-only, категории).
- `format` — human_bytes.

## oswam-tui (ratatui)
- `app` — состояние, навигация, группировка; `set_result` сортирует записи категории по размеру
  (неизвестный размер — в конец) и ставит рекомендованный выбор.
- `selection` — `is_recommended` (Safe, удаляемое, не `permanent_only`: то, что нельзя положить
  в Корзину, пользователь отмечает сам — «в Корзину» по умолчанию не должно молча означать
  «безвозвратно») и `is_bulk_pick` (удаляемое, риск ≤ Caution, не `permanent_only`) — Space на
  категории берёт только его: `Danger` и «только безвозвратно» отмечаются лишь поштучно.
  `a` — вернуть рекомендованное, `n` — снять всё, Enter — категория → файлы / выбрать файл.
- `breakdown` — сводка категории: выбрано/доступно в байтах, доли по риску (выбрано N/M, сколько
  «только поштучно»), только-просмотр. `reclaimable_total` — «из N» в action bar считается
  по записям, а не по `ScanResult.total_bytes`.
- `sidebar` — строка категории `[~] g ▲ Имя  выбрано / доступно` (компактные единицы, имя режется
  «…», размеры всегда видны); категория без удаляемого — `ⓘ размер`.
- `overview` — описание категории при фокусе на сайдбаре (вместо записи под курсором).
- `outcome` — фазы удаления/итога; `set_purged` переносит байты «в Корзине» в «освобождено»,
  не тронутые дочисткой — в `kept_in_trash` («Остались в Корзине»), ошибки — в `failed`.
- `input` — обработка `Key` (vim+стандарт).
- `theme` — палитра dark/light, цвет+символ риска.
- `detect` — авто-детект темы (COLORFGBG).
- `event` — маппинг crossterm → `Key`.
- `render`/`panels` — layout (сайдбар/описание/таблица/action) + help-оверлей; snapshot-тесты.
- `describe` — тексты «что это / почему / пустая категория» по категории и риску;
  `system_data_note` — тексты записей `system-data` (по пути и `CleanupKind`).
- `explain` — тексты про конкретную запись: действие, строка размера (`не менее X`, доля в общих
  файлах, «размер заранее неизвестен»), подсказка выбора. Оба покрыты unit-тестами.
- `query` — производные выборки (`selected_entries`, `selected_total_bytes`,
  `selected_unknown_size`, порядок категорий).
- `modals` — подтверждение (Корзина освобождает место только после очистки; «+N без оценки
  размера») и итог (обработано / в Корзине / частично / не тронуто / не удалось); если этот
  запуск что-то положил в Корзину — `e` дочищает ровно это.
- `run` — terminal event loop; `Jobs { scan, delete, purge }` — фоновые раннеры.

## oswam-cli
- `cli` — clap (сабкоманды scan/clean, default → TUI).
- `extras` — `complete(result, …)` после скана: тонкая обвязка над `runtimes::find` и
  `sysdata::absorb` (реальные `RealFs`, `run_within` с лимитом 10 с, `std::fs`); под sudo
  ещё `root_caches` и снимки Time Machine. `progress_message` зависит от `is_root()`.
- `context` — сборка окружения (`sudo::user_home`) + `run_scan` на реальной ФС + снимки TM.
- `perform` — `Runner { fs, config, home, elevated }` → `Outcome { manifest, failures }`:
  исполнение удаления (fs + native), гвард строится на каждую запись отдельно.
- `output` — печать скана/итога (`summary_lines`: в dry-run итог формулируется в
  сослагательном наклонении — превью не отчитывается о том, чего не делало).
- `commands` — хендлеры scan/clean/tui.
- `runners` — раннеры TUI: удаление (собирает `trashed` из отчётов) и дочистка Корзины
  (`purge_trashed`, `Guard` без root).

## Поток данных
registry → expand(~) → meta(FsOps) → facts → classify → measure → ScanResult →
(TUI выбор | CLI select) → Guard(per-entry) → Deleter::remove/docker → DeleteReport → manifest →
summary (отдельно «освобождено сразу», «в Корзине», «не удалось»).
