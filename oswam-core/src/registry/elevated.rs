use super::native;
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

pub fn category() -> Category {
    Category {
        id: "elevated",
        name: "Системное (только sudo)",
        glyph: "⚙",
        targets: vec![
            Target::new(
                "/Library/Developer/CoreSimulator/Caches/dyld",
                DeleteContents,
                Caution,
            )
            .needing_root(),
            Target::enumerated("/Library/Logs", DeleteContents, Caution).needing_root(),
            Target::new("/Library/Developer/CommandLineTools", DeletePath, Caution).needing_root(),
            Target::new(
                "/Library/Application Support/CrashReporter",
                DeleteContents,
                Safe,
            )
            .needing_root(),
            native(
                "Единый системный лог (log erase --all)",
                &[&["log", "config", "--status"]],
                &["log", "erase", "--all"],
                Caution,
            )
            .needing_root(),
        ],
    }
}
