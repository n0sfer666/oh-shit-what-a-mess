use crate::risk::RiskLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupKind {
    DeleteContents,
    DeletePath,
    NativeCommand,
    InfoOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeSpec {
    pub estimate: Vec<String>,
    pub clean: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub path: String,
    pub kind: CleanupKind,
    pub risk: RiskLevel,
    pub native: Option<NativeSpec>,
    pub enumerate: bool,
}

impl Target {
    fn new(path: &str, kind: CleanupKind, risk: RiskLevel) -> Self {
        Self {
            path: path.to_string(),
            kind,
            risk,
            native: None,
            enumerate: false,
        }
    }

    fn enumerated(path: &str, kind: CleanupKind, risk: RiskLevel) -> Self {
        Self {
            enumerate: true,
            ..Self::new(path, kind, risk)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: &'static str,
    pub name: &'static str,
    pub glyph: &'static str,
    pub targets: Vec<Target>,
}

fn native(label: &str, estimate: &[&str], clean: &[&str], risk: RiskLevel) -> Target {
    Target {
        path: label.to_string(),
        kind: CleanupKind::NativeCommand,
        risk,
        native: Some(NativeSpec {
            estimate: estimate.iter().map(|s| s.to_string()).collect(),
            clean: clean.iter().map(|s| s.to_string()).collect(),
        }),
        enumerate: false,
    }
}

pub fn builtin_categories() -> Vec<Category> {
    use CleanupKind::*;
    use RiskLevel::*;
    vec![
        Category {
            id: "system",
            name: "Системный мусор",
            glyph: "🧹",
            targets: vec![
                Target::enumerated("~/Library/Caches", DeleteContents, Safe),
                Target::new("~/Library/Logs", DeleteContents, Safe),
                Target::new("~/.Trash", DeleteContents, Safe),
                Target::new(
                    "~/Library/Caches/com.apple.QuickLook.thumbnailcache",
                    DeletePath,
                    Safe,
                ),
            ],
        },
        Category {
            id: "dev",
            name: "Dev-окружение",
            glyph: "🛠",
            targets: vec![
                Target::new("~/.npm", DeleteContents, Safe),
                Target::new("~/.cache", DeleteContents, Safe),
                Target::new("~/Library/Developer/Xcode/DerivedData", DeletePath, Safe),
                Target::enumerated(
                    "~/Library/Developer/Xcode/iOS DeviceSupport",
                    DeletePath,
                    Caution,
                ),
                Target::enumerated("~/Library/Developer/Xcode/Archives", DeletePath, Caution),
                Target::new(
                    "~/Library/Developer/CoreSimulator/Caches",
                    DeleteContents,
                    Safe,
                ),
                native(
                    "Docker (docker system prune)",
                    &["docker", "system", "df"],
                    &["docker", "system", "prune", "-f"],
                    Caution,
                ),
                native(
                    "Xcode: недоступные симуляторы",
                    &["true"],
                    &["xcrun", "simctl", "delete", "unavailable"],
                    Safe,
                ),
            ],
        },
        Category {
            id: "big-data",
            name: "Большие данные (инфо)",
            glyph: "📦",
            targets: vec![Target::enumerated(
                "~/Library/Application Support/MobileSync/Backup",
                InfoOnly,
                Caution,
            )],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_v2_categories() {
        let ids: Vec<&str> = builtin_categories().iter().map(|c| c.id).collect();
        assert_eq!(ids, vec!["system", "dev", "big-data"]);
    }

    #[test]
    fn caches_are_enumerated() {
        let sys = builtin_categories()
            .into_iter()
            .find(|c| c.id == "system")
            .unwrap();
        let caches = sys
            .targets
            .iter()
            .find(|t| t.path == "~/Library/Caches")
            .unwrap();
        assert!(caches.enumerate);
        assert_eq!(caches.kind, CleanupKind::DeleteContents);
    }

    #[test]
    fn docker_is_native_and_never_touches_raw() {
        let dev = builtin_categories()
            .into_iter()
            .find(|c| c.id == "dev")
            .unwrap();
        let docker = dev
            .targets
            .iter()
            .find(|t| {
                t.native
                    .as_ref()
                    .is_some_and(|s| s.clean.first().map(String::as_str) == Some("docker"))
            })
            .unwrap();
        let spec = docker.native.as_ref().unwrap();
        assert_eq!(spec.clean, vec!["docker", "system", "prune", "-f"]);
        assert!(spec.clean.iter().all(|a| !a.contains("raw")));
    }

    #[test]
    fn big_data_is_info_only() {
        let bd = builtin_categories()
            .into_iter()
            .find(|c| c.id == "big-data")
            .unwrap();
        assert!(bd.targets.iter().all(|t| t.kind == CleanupKind::InfoOnly));
    }
}
