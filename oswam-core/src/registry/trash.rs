use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

pub fn category() -> Category {
    Category {
        id: "trash",
        name: "Корзина",
        glyph: "🗑",
        targets: vec![Target::new("~/.Trash", DeleteContents, Danger)],
    }
}
