use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

pub fn category() -> Category {
    Category {
        id: "system",
        name: "Системный мусор",
        glyph: "🧹",
        targets: vec![
            Target::enumerated("~/Library/Caches", DeleteContents, Safe),
            Target::new("~/Library/Logs", DeleteContents, Safe),
            Target::new("~/.Trash", DeleteContents, Danger),
            Target::new("~/Library/Application Support/Caches", DeleteContents, Safe),
            Target::new("~/Library/Saved Application State", DeleteContents, Caution),
        ],
    }
}
