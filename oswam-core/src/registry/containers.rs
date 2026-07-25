use super::native_filtered;
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

const DF: [&str; 5] = [
    "docker",
    "system",
    "df",
    "--format",
    "{{.Type}}\t{{.Reclaimable}}",
];

pub fn category() -> Category {
    Category {
        id: "containers",
        name: "Контейнеры",
        glyph: "🐳",
        targets: vec![
            native_filtered(
                "Docker: неиспользуемые образы (docker image prune -af)",
                &[&["docker", "info"]],
                &DF,
                &["docker", "image", "prune", "-af"],
                &["Images"],
                Caution,
            ),
            native_filtered(
                "Docker: остановленные контейнеры (docker container prune -f)",
                &[&["docker", "info"]],
                &DF,
                &["docker", "container", "prune", "-f"],
                &["Containers"],
                Caution,
            ),
            native_filtered(
                "Docker: кэш сборки (docker builder prune -af)",
                &[&["docker", "info"]],
                &DF,
                &["docker", "builder", "prune", "-af"],
                &["Build Cache"],
                Caution,
            ),
            native_filtered(
                "Docker: неиспользуемые тома (docker volume prune -af)",
                &[&["docker", "info"]],
                &DF,
                &["docker", "volume", "prune", "-af"],
                &["Local Volumes"],
                Danger,
            ),
            Target::new("~/.local/share/containers/podman", InfoOnly, Danger),
        ],
    }
}
