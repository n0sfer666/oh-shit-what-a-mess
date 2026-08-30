use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

pub fn category() -> Category {
    let base = "~/Library/Application Support";
    Category {
        id: "big-data",
        name: "Большие данные (инфо/риск)",
        glyph: "📦",
        targets: vec![
            Target::enumerated(
                "~/Library/Application Support/MobileSync/Backup",
                InfoOnly,
                Danger,
            ),
            Target::new(
                &format!("{base}/Steam/steamapps/downloading"),
                DeleteContents,
                Caution,
            ),
            Target::enumerated(&format!("{base}/Steam/steamapps/common"), InfoOnly, Danger),
            Target::new(&format!("{base}/Claude"), InfoOnly, Danger),
            Target::new(&format!("{base}/Claude/vm_bundles"), InfoOnly, Danger),
            Target::new(&format!("{base}/Google"), InfoOnly, Danger),
            Target::new(
                "~/Library/Group Containers/6N38VWS5BX.ru.keepcoder.Telegram",
                InfoOnly,
                Danger,
            ),
        ],
    }
}
