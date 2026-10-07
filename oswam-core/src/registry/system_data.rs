use super::native;
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

pub const ID: &str = "system-data";

pub fn category() -> Category {
    Category {
        id: ID,
        name: "System Data (macOS)",
        glyph: "💽",
        targets: vec![
            Target::enumerated("~/Library/Caches", DeleteContents, Safe),
            Target::new("~/Library/Logs", DeleteContents, Safe),
            Target::new("~/Library/Application Support/Caches", DeleteContents, Safe),
            Target::new("~/Library/Saved Application State", DeleteContents, Caution),
            Target::new("/System/Volumes/VM", InfoOnly, Danger),
            Target::new("/private/var/vm", InfoOnly, Danger),
            Target::new("/private/var/folders", InfoOnly, Danger),
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
            Target::new("/System/Volumes/Data/.Spotlight-V100", InfoOnly, Danger).needing_root(),
            Target::new(
                "/System/Volumes/Data/.DocumentRevisions-V100",
                InfoOnly,
                Danger,
            )
            .needing_root(),
        ],
    }
}
