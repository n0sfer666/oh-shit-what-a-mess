use oswam_core::category::CleanupKind;
use oswam_core::format::human_bytes;
use oswam_core::scan::ScanEntry;
use oswam_core::select::is_deletable;

pub fn action_text(entry: &ScanEntry) -> String {
    match entry.kind {
        CleanupKind::DeleteContents if entry.permanent_only => {
            "Удалит содержимое безвозвратно — Корзина здесь недоступна".into()
        }
        CleanupKind::DeletePath if entry.permanent_only => {
            "Удалит каталог целиком безвозвратно — Корзина здесь недоступна".into()
        }
        CleanupKind::DeleteContents => "Удалит содержимое (Корзина/безвозвратно по выбору)".into(),
        CleanupKind::DeletePath => "Удалит каталог целиком".into(),
        CleanupKind::NativeCommand => match &entry.native {
            Some(spec) => format!("Выполнит: {}", spec.clean.join(" ")),
            None => "Нативная команда".into(),
        },
        CleanupKind::InfoOnly => "Только информация — удаление недоступно".into(),
    }
}

pub fn size_line(entry: &ScanEntry) -> String {
    let size = human_bytes(entry.physical_bytes);
    if entry.size_unknown && !is_deletable(entry) {
        return "Занимает: размер неизвестен (удаление недоступно)".into();
    }
    if !is_deletable(entry) {
        return format!("Занимает: {size} (удаление недоступно)");
    }
    if entry.size_unknown {
        return "Освободится: размер заранее неизвестен".into();
    }
    let shared = shared_note(entry);
    if entry.partial {
        return format!("Освободится: не менее {size} (часть содержимого недоступна){shared}");
    }
    format!("Освободится: {size}{shared}")
}

fn shared_note(entry: &ScanEntry) -> String {
    if entry.shared_bytes == 0 {
        return String::new();
    }
    format!(
        " · из них {} в общих файлах — освободятся только с последней ссылкой",
        human_bytes(entry.shared_bytes)
    )
}

pub fn select_hint(entry: &ScanEntry) -> &'static str {
    if is_deletable(entry) {
        "Space — выбрать · Ctrl+P — удалить выбранное"
    } else {
        "Недоступно для выбора (только просмотр)."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oswam_core::category::NativeSpec;
    use oswam_core::risk::RiskLevel;
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
    fn a_native_without_an_estimate_says_size_unknown() {
        let mut e = entry(CleanupKind::NativeCommand, RiskLevel::Safe, 0);
        e.size_unknown = true;
        assert!(size_line(&e).contains("неизвестен"));
    }

    #[test]
    fn a_native_with_an_estimate_of_zero_is_not_called_unknown() {
        let e = entry(CleanupKind::NativeCommand, RiskLevel::Safe, 0);
        assert!(!size_line(&e).contains("неизвестен"));
    }

    #[test]
    fn shared_bytes_are_flagged_as_not_freed_alone() {
        let mut e = entry(CleanupKind::DeletePath, RiskLevel::Safe, 4096);
        e.shared_bytes = 2048;
        let line = size_line(&e);
        assert!(line.contains("в общих файлах"), "{line}");
    }
}
