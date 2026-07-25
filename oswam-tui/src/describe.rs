use oswam_core::category::CleanupKind;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::ScanEntry;

pub fn why_text(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => "Безопасно: регенерируемый кэш, приложение пересоздаст.",
        RiskLevel::Caution => "Осторожно: возможно используется процессом или частично нужное.",
        RiskLevel::Danger => "Опасно: необратимо и не восстановится само — проверьте, что внутри.",
        RiskLevel::Never => "Никогда: системное/защищённое, удаление заблокировано.",
    }
}

pub fn what_is_it(category_id: &str, entry: &ScanEntry) -> &'static str {
    if category_id == "snapshots" {
        return match entry.kind {
            CleanupKind::InfoOnly => {
                "Самый свежий локальный снимок Time Machine — единственная точка отката, поэтому не удаляется."
            }
            _ => "Локальные снимки Time Machine — удаляются все, кроме самого свежего.",
        };
    }
    if entry.kind == CleanupKind::InfoOnly {
        if category_id == "simulators" {
            return "Комплект файлов, часть которого защищена: удалить можно только целиком, поэтому показано для справки.";
        }
        if category_id == "system-caches" {
            return "Системный кэш под защитой (SIP или ваш protected_paths) — показан для справки, удалять его OSWaM не станет.";
        }
        return "Данные приложения — показаны для справки, чистить только вручную в самом приложении.";
    }
    if entry.risk == RiskLevel::Danger {
        return danger_text(category_id);
    }
    if category_id == "elevated" && entry.kind == CleanupKind::NativeCommand {
        return "Единый системный лог — очистка необратима: диагностика прошлых сбоев пропадёт.";
    }
    match category_id {
        "system" => "Системный кэш/мусор — ОС и приложения пересоздадут при работе.",
        "dev" => "Dev-кэш или артефакт — инструмент перекачает/пересоберёт при следующем запуске.",
        "simulators" => {
            "Образ симулятора/эмулятора — вернётся после повторной загрузки в Xcode/Android Studio."
        }
        "containers" => "Слой или кэш Docker — пересоберётся/перекачается при следующем build.",
        "browsers" => {
            "Кэш браузера — перезагрузится сам; пароли, история и вкладки не затрагиваются."
        }
        "projects" => "Build-артефакт проекта — вернётся после сборки (cargo build / npm install).",
        "big-data" => "Крупные данные — освободят много места, но потребуют повторной загрузки.",
        "elevated" => {
            "Системный кэш/лог вне домашней папки — доступен только при запуске под sudo."
        }
        "snapshots" => "Локальный снимок Time Machine — удаляется, кроме самого свежего.",
        _ => "Кэш/артефакт — обычно безопасно удалять.",
    }
}

fn danger_text(category_id: &str) -> &'static str {
    match category_id {
        "system-caches" => {
            "Общесистемный кэш вне домашней папки: приложения его пересоздадут, но удаление идёт под root, минуя Корзину, и может задеть работающие процессы."
        }
        "simulators" => {
            "Эмулятор со СВОИМ состоянием: установленные приложения, аккаунты и снапшоты пропадут навсегда."
        }
        "containers" => {
            "Именованные тома Docker — внутри могут быть БД и данные сервисов. Восстановления нет."
        }
        "big-data" => "Пользовательские данные/игры — вернуть можно только повторной загрузкой.",
        "dev" => "Тулчейн целиком — понадобится переустановка (rustup/n install).",
        _ => "Необратимое удаление пользовательских данных — убедитесь, что копия есть.",
    }
}

pub fn empty_note(category_id: &str, elevated: bool) -> &'static str {
    match (category_id, elevated) {
        ("elevated", false) => "Раздел доступен только под sudo — запустите `sudo oswam`.",
        ("elevated", true) => "Системные кэши и логи уже чисты — удалять нечего.",
        ("", _) => "Нет данных для отображения.",
        _ => "В этой категории ничего не найдено — чистить нечего.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oswam_core::category::NativeSpec;
    use std::path::PathBuf;

    fn entry(kind: CleanupKind, risk: RiskLevel, bytes: u64) -> ScanEntry {
        ScanEntry {
            display: "x".into(),
            path: PathBuf::from("/x"),
            kind,
            risk,
            physical_bytes: bytes,
            native: matches!(kind, CleanupKind::NativeCommand).then(NativeSpec::default),
            companions: Vec::new(),
            shared_bytes: 0,
            size_unknown: false,
            partial: false,
            needs_root: false,
            permanent_only: false,
        }
    }

    #[test]
    fn new_categories_have_their_own_text() {
        let e = entry(CleanupKind::DeletePath, RiskLevel::Caution, 1);
        let fallback = what_is_it("unknown", &e);
        for id in ["simulators", "containers", "elevated"] {
            assert_ne!(what_is_it(id, &e), fallback, "{id}");
        }
    }

    #[test]
    fn danger_never_reads_as_safe() {
        let e = entry(CleanupKind::DeletePath, RiskLevel::Danger, 1);
        for id in ["simulators", "containers", "big-data", "dev", "unknown"] {
            let text = what_is_it(id, &e);
            assert!(!text.contains("безопасно"), "{id}: {text}");
        }
    }

    #[test]
    fn empty_elevated_category_explains_sudo() {
        assert!(empty_note("elevated", false).contains("sudo"));
        assert!(!empty_note("elevated", true).contains("sudo"));
        assert_ne!(empty_note("dev", false), empty_note("elevated", false));
    }

    #[test]
    fn a_protected_system_cache_is_not_called_app_data() {
        let e = entry(CleanupKind::InfoOnly, RiskLevel::Never, 1);
        let text = what_is_it("system-caches", &e);
        assert!(!text.contains("Данные приложения"), "{text}");
        assert!(text.contains("защит"), "{text}");
    }

    #[test]
    fn system_log_erase_warns_about_irreversibility() {
        let e = entry(CleanupKind::NativeCommand, RiskLevel::Caution, 0);
        assert!(what_is_it("elevated", &e).contains("необратима"));
    }
}
