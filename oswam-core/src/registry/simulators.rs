use super::native;
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

const SIMCTL: [&[&str]; 2] = [&["xcode-select", "-p"], &["xcrun", "--find", "simctl"]];

pub fn category() -> Category {
    Category {
        id: "simulators",
        name: "Симуляторы и эмуляторы",
        glyph: "🧪",
        targets: vec![
            Target::new(
                "~/Library/Developer/CoreSimulator/Caches",
                DeleteContents,
                Safe,
            ),
            Target::grouped("~/.android/avd", DeletePath, Danger),
            Target::enumerated("~/Library/Android/sdk/system-images", DeletePath, Caution),
            Target::enumerated("~/Library/Android/sdk/ndk", DeletePath, Caution),
            Target::new("~/Library/Android/sdk/sources", DeletePath, Caution),
            native(
                "Xcode: недоступные симуляторы",
                &SIMCTL,
                &["xcrun", "simctl", "delete", "unavailable"],
                Safe,
            ),
            native(
                "Xcode: недоступные iOS runtimes",
                &SIMCTL,
                &["xcrun", "simctl", "runtime", "delete", "unavailable"],
                Caution,
            ),
            native(
                "Xcode: iOS runtimes без использования 180 дней",
                &SIMCTL,
                &[
                    "xcrun",
                    "simctl",
                    "runtime",
                    "delete",
                    "--notUsedSinceDays",
                    "180",
                ],
                Caution,
            ),
        ],
    }
}
