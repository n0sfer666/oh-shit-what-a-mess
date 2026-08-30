use super::{native, native_filtered};
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

const SIMCTL: [&[&str]; 2] = [&["xcode-select", "-p"], &["xcrun", "--find", "simctl"]];

const RUNTIME_LIST: [&str; 4] = ["xcrun", "simctl", "runtime", "list"];

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
            Target::new(
                "~/Library/Developer/CoreSimulator/Devices",
                InfoOnly,
                Danger,
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
                "Xcode: все симуляторы (simctl delete all)",
                &SIMCTL,
                &["xcrun", "simctl", "delete", "all"],
                Danger,
            ),
            native_filtered(
                "Xcode: все iOS runtimes (simctl runtime delete all)",
                &SIMCTL,
                &RUNTIME_LIST,
                &["xcrun", "simctl", "runtime", "delete", "all"],
                &["Total Disk Images"],
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
