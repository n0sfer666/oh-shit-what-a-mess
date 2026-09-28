use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

const ROOTS: [&str; 7] = [
    "~/Developer",
    "~/Projects",
    "~/dev",
    "~/_dev",
    "~/src",
    "~/code",
    "~/work",
];

const ARTIFACT_DIRS: [&str; 11] = [
    "target",
    "build",
    "node_modules",
    ".next",
    "build-*",
    "cmake-build-*",
    "DerivedData",
    ".turbo",
    ".parcel-cache",
    ".dart_tool",
    "__pycache__",
];

pub fn category() -> Category {
    Category {
        id: "projects",
        name: "Dev-проекты (пересборка)",
        glyph: "📁",
        targets: ROOTS
            .iter()
            .map(|root| Target::discovered(root, ARTIFACT_DIRS.to_vec(), 4, DeletePath, Caution))
            .collect(),
    }
}
