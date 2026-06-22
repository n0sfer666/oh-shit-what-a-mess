use crate::risk::RiskLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupKind {
    DeleteContents,
    DeletePath,
    NativeCommand,
    InfoOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl Target {
    fn new(path: &str, kind: CleanupKind, risk: RiskLevel) -> Self {
        Self {
            path: path.to_string(),
            kind,
            risk,
            native: None,
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
                Target::new("~/Library/Caches/Yarn", DeleteContents, Safe),
                Target::new("~/Library/Caches/go-build", DeleteContents, Safe),
                Target::new("~/Library/Developer/Xcode/DerivedData", DeletePath, Safe),
                native(
                    "Docker (docker system prune)",
                    &["docker", "system", "df"],
                    &["docker", "system", "prune", "-f"],
                    Caution,
                ),
            ],
        },
        Category {
            id: "browsers",
            name: "Браузеры",
            glyph: "🌐",
            targets: vec![
                Target::new("~/Library/Caches/Google", DeleteContents, Safe),
                Target::new("~/Library/Caches/Arc", DeleteContents, Safe),
                Target::new("~/Library/Caches/Comet", DeleteContents, Safe),
                Target::new("~/Library/Caches/Yandex", DeleteContents, Safe),
                Target::new("~/Library/Caches/com.apple.Safari", DeleteContents, Safe),
            ],
        },
        Category {
            id: "big-data",
            name: "Большие данные (инфо)",
            glyph: "📦",
            targets: vec![Target::new(
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
    fn has_v1_categories() {
        let ids: Vec<&str> = builtin_categories().iter().map(|c| c.id).collect();
        assert_eq!(ids, vec!["system", "dev", "browsers", "big-data"]);
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
            .find(|t| t.kind == CleanupKind::NativeCommand)
            .unwrap();
        let spec = docker.native.as_ref().unwrap();
        assert_eq!(spec.clean, vec!["docker", "system", "prune", "-f"]);
        assert!(!docker.path.contains(".raw"));
        assert!(spec.estimate.iter().all(|a| !a.contains("raw")));
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

    #[test]
    fn cache_roots_use_delete_contents() {
        let sys = builtin_categories()
            .into_iter()
            .find(|c| c.id == "system")
            .unwrap();
        let logs = sys
            .targets
            .iter()
            .find(|t| t.path.ends_with("Logs"))
            .unwrap();
        assert_eq!(logs.kind, CleanupKind::DeleteContents);
    }
}
