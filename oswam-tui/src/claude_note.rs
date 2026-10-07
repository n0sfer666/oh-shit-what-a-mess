use oswam_core::scan::ScanEntry;

pub fn what_is_it(entry: &ScanEntry) -> &'static str {
    if entry.path.ends_with("rootfs.img") {
        return "Базовый образ Linux-VM для Cowork: Claude скачает его заново целиком при следующем запуске Cowork. Сессии (sessiondata.img) не затрагиваются.";
    }
    let parent = entry.path.parent().and_then(|p| p.file_name());
    if parent.is_some_and(|p| p == "claude-code" || p == "claude-code-vm") {
        return "Встроенная сборка Claude Code: старую можно удалять, текущую Claude скачает заново.";
    }
    "Кэш оболочки Claude (Chromium) — пересоздастся сам; логин, чаты и настройки не затрагиваются."
}

#[cfg(test)]
mod tests {
    use super::*;
    use oswam_core::category::CleanupKind;
    use oswam_core::risk::RiskLevel;
    use std::path::PathBuf;

    const ROOT: &str = "/Users/tester/Library/Application Support/Claude";

    fn at(path: &str) -> ScanEntry {
        ScanEntry {
            display: "x".into(),
            path: PathBuf::from(format!("{ROOT}/{path}")),
            kind: CleanupKind::DeletePath,
            risk: RiskLevel::Caution,
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
    fn each_kind_of_claude_entry_explains_itself() {
        assert!(what_is_it(&at("vm_bundles/claudevm.bundle/rootfs.img")).contains("Cowork"));
        assert!(what_is_it(&at("claude-code/2.1.288")).contains("Claude Code"));
        assert!(what_is_it(&at("claude-code-vm/2.1.289")).contains("Claude Code"));
        assert!(what_is_it(&at("Code Cache")).contains("Кэш"));
    }
}
