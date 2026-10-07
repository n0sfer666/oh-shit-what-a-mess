use oswam_core::category::CleanupKind;
use oswam_core::registry::SYSTEM_DATA_ID;
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
    if category_id == SYSTEM_DATA_ID {
        return crate::system_data_note::what_is_it(entry);
    }
    if entry.kind == CleanupKind::InfoOnly {
        if category_id == "simulators" {
            return if entry.path.ends_with("CoreSimulator/Devices") {
                "Устройства симулятора: вес лежит здесь, но удаляет их только simctl — команды в этой же категории."
            } else {
                "Комплект файлов, часть которого защищена: удалить можно только целиком, поэтому показано для справки."
            };
        }
        return "Данные приложения — показаны для справки, чистить только вручную в самом приложении.";
    }
    if entry.display.contains("runtime delete all") {
        return "Все iOS-рантаймы: показан их полный вес, но занятый запущенным симулятором рантайм остаётся — освободится не больше показанного.";
    }
    if entry.path.ends_with(".vscode/extensions") {
        return "Расширения VS Code вместе с их списком (extensions.json): что было установлено, восстанавливать придётся вручную.";
    }
    if entry.risk == RiskLevel::Danger {
        return danger_text(category_id);
    }
    if category_id == "claude" {
        return crate::claude_note::what_is_it(entry);
    }
    match category_id {
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
        "snapshots" => "Локальный снимок Time Machine — удаляется, кроме самого свежего.",
        _ => "Кэш/артефакт — обычно безопасно удалять.",
    }
}

fn danger_text(category_id: &str) -> &'static str {
    match category_id {
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

pub fn empty_note(category_id: &str) -> &'static str {
    match category_id {
        "" => "Нет данных для отображения.",
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
            blocking_app: None,
            blocked: false,
        }
    }

    #[test]
    fn new_categories_have_their_own_text() {
        let e = entry(CleanupKind::DeletePath, RiskLevel::Caution, 1);
        let fallback = what_is_it("unknown", &e);
        for id in ["simulators", "containers", SYSTEM_DATA_ID] {
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
    fn simulator_devices_are_not_called_a_protected_bundle() {
        let mut e = entry(CleanupKind::InfoOnly, RiskLevel::Caution, 1);
        e.path = PathBuf::from("/Users/tester/Library/Developer/CoreSimulator/Devices");
        let text = what_is_it("simulators", &e);
        assert!(text.contains("simctl"), "{text}");
        assert!(!text.contains("защищена"), "{text}");
        let other = entry(CleanupKind::InfoOnly, RiskLevel::Caution, 1);
        assert_ne!(what_is_it("simulators", &other), text);
    }

    #[test]
    fn losing_the_extension_list_is_spelled_out() {
        let mut e = entry(CleanupKind::DeleteContents, RiskLevel::Danger, 1);
        e.path = PathBuf::from("/Users/tester/.vscode/extensions");
        let text = what_is_it("dev", &e);
        assert!(text.contains("вручную"), "{text}");
        assert!(!text.contains("перекачает"), "{text}");
    }

    #[test]
    fn a_wholesale_runtime_delete_promises_no_more_than_it_frees() {
        let mut e = entry(CleanupKind::NativeCommand, RiskLevel::Caution, 39);
        e.display = "Xcode: все iOS runtimes (simctl runtime delete all)".into();
        let text = what_is_it("simulators", &e);
        assert!(text.contains("не больше показанного"), "{text}");
    }
}
