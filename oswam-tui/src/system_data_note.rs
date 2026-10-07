use oswam_core::category::CleanupKind;
use oswam_core::registry::SYSTEM_DATA_ID;
use oswam_core::risk::RiskLevel;
use oswam_core::runtimes::ASSETS;
use oswam_core::scan::ScanEntry;

pub fn what_is_it(entry: &ScanEntry) -> &'static str {
    let system_cache = entry.path.starts_with("/Library/Caches");
    if entry.kind == CleanupKind::InfoOnly {
        return if entry.path.starts_with(ASSETS) {
            "Осиротевший рантайм iOS-симулятора: simctl его не видит и удалить не может, сама macOS его тоже не уберёт. Штатного способа нет — только вручную из Recovery с выключенным SIP."
        } else if system_cache {
            "Системный кэш под защитой (SIP или ваш protected_paths) — показан для справки, удалять его OSWaM не станет."
        } else {
            "Служебные файлы macOS (подкачка, временные файлы, индекс Spotlight, версии документов) — показаны, чтобы было видно, из чего складывается System Data; ими управляет сама система."
        };
    }
    if erases_the_system_log(entry) {
        return "Единый системный лог — очистка необратима: диагностика прошлых сбоев пропадёт.";
    }
    if entry.path.ends_with("Saved Application State") {
        return "Сохранённое состояние окон: после очистки приложения откроются без восстановленных окон и вкладок.";
    }
    if entry.path.ends_with("Developer/CommandLineTools") {
        return "Command Line Tools целиком: без Xcode.app это выключает git, clang и make до `xcode-select --install` (~1.5 GB заново).";
    }
    if system_cache && entry.risk == RiskLevel::Danger {
        return "Общесистемный кэш вне домашней папки: приложения его пересоздадут, но удаление идёт под root, минуя Корзину, и может задеть работающие процессы.";
    }
    if entry.needs_root {
        return "Системный кэш/лог вне домашней папки — доступен только при запуске под sudo.";
    }
    "Системный кэш/мусор — ОС и приложения пересоздадут при работе."
}

pub fn sudo_hint(category_id: &str, elevated: bool) -> Option<&'static str> {
    (category_id == SYSTEM_DATA_ID && !elevated).then_some(
        "Часть System Data (/Library, системные логи, Spotlight) видна только под `sudo oswam`.",
    )
}

fn erases_the_system_log(entry: &ScanEntry) -> bool {
    entry
        .native
        .as_ref()
        .is_some_and(|n| n.clean.iter().take(2).eq(["log", "erase"].iter()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oswam_core::category::NativeSpec;
    use std::path::PathBuf;

    fn at(path: &str, kind: CleanupKind, risk: RiskLevel) -> ScanEntry {
        ScanEntry {
            display: "x".into(),
            path: PathBuf::from(path),
            kind,
            risk,
            physical_bytes: 1,
            native: None,
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
    fn an_orphan_runtime_says_why_oswam_cannot_remove_it() {
        let e = at(
            &format!("{ASSETS}/old.asset"),
            CleanupKind::InfoOnly,
            RiskLevel::Never,
        );
        let text = what_is_it(&e);
        assert!(text.contains("simctl") && text.contains("SIP"), "{text}");
    }

    #[test]
    fn a_protected_system_cache_is_not_called_app_data() {
        let e = at("/Library/Caches/x", CleanupKind::InfoOnly, RiskLevel::Never);
        let text = what_is_it(&e);
        assert!(text.contains("защит"), "{text}");
    }

    #[test]
    fn macos_owned_space_is_not_offered_as_junk() {
        let e = at(
            "/System/Volumes/VM",
            CleanupKind::InfoOnly,
            RiskLevel::Never,
        );
        let text = what_is_it(&e);
        assert!(text.contains("управляет сама система"), "{text}");
        assert!(!text.contains("пересоздадут"), "{text}");
    }

    fn native(clean: &[&str]) -> ScanEntry {
        let mut e = at("x", CleanupKind::NativeCommand, RiskLevel::Caution);
        e.native = Some(NativeSpec {
            clean: clean.iter().map(|s| s.to_string()).collect(),
            ..NativeSpec::default()
        });
        e
    }

    #[test]
    fn system_log_erase_warns_about_irreversibility() {
        assert!(what_is_it(&native(&["log", "erase", "--all"])).contains("необратима"));
    }

    #[test]
    fn another_native_is_not_described_as_the_system_log() {
        assert!(!what_is_it(&native(&["tmutil", "thinlocalsnapshots"])).contains("лог"));
    }

    #[test]
    fn saved_window_state_is_not_called_a_cache() {
        let e = at(
            "/Users/tester/Library/Saved Application State",
            CleanupKind::DeleteContents,
            RiskLevel::Caution,
        );
        let text = what_is_it(&e);
        assert!(text.contains("окон") && !text.contains("кэш"), "{text}");
    }

    #[test]
    fn system_data_without_sudo_says_part_of_it_is_hidden() {
        assert!(sudo_hint(SYSTEM_DATA_ID, false).is_some_and(|t| t.contains("sudo")));
        assert!(sudo_hint(SYSTEM_DATA_ID, true).is_none());
        assert!(sudo_hint("dev", false).is_none());
    }

    #[test]
    fn a_toolchain_is_not_described_as_a_cache() {
        let mut e = at(
            "/Library/Developer/CommandLineTools",
            CleanupKind::DeletePath,
            RiskLevel::Caution,
        );
        e.needs_root = true;
        let text = what_is_it(&e);
        assert!(text.contains("xcode-select"), "{text}");
        assert!(!text.contains("кэш"), "{text}");
    }

    #[test]
    fn a_root_wide_cache_warns_it_skips_the_trash() {
        let e = at(
            "/Library/Caches/x",
            CleanupKind::DeleteContents,
            RiskLevel::Danger,
        );
        assert!(what_is_it(&e).contains("минуя Корзину"));
    }

    #[test]
    fn a_sudo_only_cache_says_so() {
        let mut e = at(
            "/Library/Logs/x",
            CleanupKind::DeleteContents,
            RiskLevel::Caution,
        );
        e.needs_root = true;
        assert!(what_is_it(&e).contains("sudo"));
    }
}
