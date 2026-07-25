use super::{native, native_at};
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

pub fn category() -> Category {
    Category {
        id: "dev",
        name: "Dev-окружение",
        glyph: "🛠",
        targets: vec![
            native_at(
                "~/.npm/_cacache",
                &[&["npm", "--version"]],
                &["npm", "cache", "clean", "--force"],
                Caution,
            ),
            Target::new("~/.yarn/berry/cache", DeleteContents, Caution),
            Target::new("~/.cache", DeleteContents, Caution),
            Target::new("~/.gradle/caches", DeleteContents, Caution),
            Target::new("~/.gradle/daemon", DeleteContents, Safe),
            Target::new("~/.gradle/wrapper", DeleteContents, Caution),
            Target::new("~/.cargo/registry/cache", DeleteContents, Caution),
            Target::new("~/.bun/install/cache", DeleteContents, Caution),
            Target::new("~/.pub-cache", DeleteContents, Caution),
            Target::new("~/.nuget/packages", DeleteContents, Caution),
            Target::new("~/.konan", DeleteContents, Caution),
            Target::new("~/.local/share/nvim/mason", DeleteContents, Caution),
            Target::new("~/.serena/language_servers", DeleteContents, Caution),
            Target::new("~/Library/Developer/Xcode/DerivedData", DeletePath, Safe),
            Target::enumerated(
                "~/Library/Developer/Xcode/iOS DeviceSupport",
                DeletePath,
                Caution,
            ),
            Target::enumerated("~/Library/Developer/Xcode/Archives", DeletePath, Caution),
            Target::enumerated("~/.rustup/toolchains", DeletePath, Caution),
            Target::enumerated("/usr/local/n/versions/node", DeletePath, Caution),
            native(
                "pnpm store (pnpm store prune)",
                &[&["pnpm", "store", "path"]],
                &["pnpm", "store", "prune"],
                Safe,
            ),
        ],
    }
}
