use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

const CACHE_DIRS: [&str; 9] = [
    "Cache",
    "Code Cache",
    "GPUCache",
    "CacheStorage",
    "ShaderCache",
    "GrShaderCache",
    "DawnGraphiteCache",
    "DawnWebGPUCache",
    "OptGuideOnDeviceModel",
];

const ROOTS: [&str; 8] = [
    "Google/Chrome",
    "Arc/User Data",
    "Comet",
    "Yandex/YandexBrowser",
    "BraveSoftware/Brave-Browser",
    "Microsoft Edge",
    "Vivaldi",
    "Chromium",
];

pub fn category() -> Category {
    Category {
        id: "browsers",
        name: "Браузеры (только кэш)",
        glyph: "🌐",
        targets: ROOTS
            .iter()
            .map(|root| {
                Target::discovered(
                    &format!("~/Library/Application Support/{root}"),
                    CACHE_DIRS.to_vec(),
                    5,
                    DeletePath,
                    Caution,
                )
            })
            .collect(),
    }
}
