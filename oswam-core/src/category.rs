use crate::risk::RiskLevel;
use serde::{Deserialize, Serialize};

pub use crate::registry::builtin_categories;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupKind {
    DeleteContents,
    DeletePath,
    NativeCommand,
    InfoOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NativeSpec {
    pub estimate: Vec<String>,
    pub clean: Vec<String>,
    #[serde(default)]
    pub estimate_filter: Vec<String>,
    #[serde(default)]
    pub probe: Vec<Vec<String>>,
    #[serde(default)]
    pub privileged: bool,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub path: String,
    pub kind: CleanupKind,
    pub risk: RiskLevel,
    pub native: Option<NativeSpec>,
    pub enumerate: bool,
    pub discover: Option<Vec<&'static str>>,
    pub depth: usize,
    pub needs_root: bool,
    pub group_by_stem: bool,
    pub measured: bool,
}

impl Target {
    pub fn new(path: &str, kind: CleanupKind, risk: RiskLevel) -> Self {
        Self {
            path: path.to_string(),
            kind,
            risk,
            native: None,
            enumerate: false,
            discover: None,
            depth: 0,
            needs_root: false,
            group_by_stem: false,
            measured: false,
        }
    }

    pub fn enumerated(path: &str, kind: CleanupKind, risk: RiskLevel) -> Self {
        Self {
            enumerate: true,
            ..Self::new(path, kind, risk)
        }
    }

    pub fn grouped(path: &str, kind: CleanupKind, risk: RiskLevel) -> Self {
        Self {
            group_by_stem: true,
            ..Self::enumerated(path, kind, risk)
        }
    }

    pub fn discovered(
        root: &str,
        names: Vec<&'static str>,
        depth: usize,
        kind: CleanupKind,
        risk: RiskLevel,
    ) -> Self {
        Self {
            discover: Some(names),
            depth,
            ..Self::new(root, kind, risk)
        }
    }

    pub fn needing_root(mut self) -> Self {
        self.needs_root = true;
        if let Some(spec) = self.native.as_mut() {
            spec.privileged = true;
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: &'static str,
    pub name: &'static str,
    pub glyph: &'static str,
    pub targets: Vec<Target>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_flags() {
        let plain = Target::new("~/x", CleanupKind::DeleteContents, RiskLevel::Safe);
        assert!(!plain.enumerate);
        assert!(plain.discover.is_none());

        let enumd = Target::enumerated("~/y", CleanupKind::DeletePath, RiskLevel::Caution);
        assert!(enumd.enumerate);

        let disc = Target::discovered(
            "~/_dev",
            vec!["target", "node_modules"],
            4,
            CleanupKind::DeletePath,
            RiskLevel::Caution,
        );
        assert_eq!(
            disc.discover.as_deref(),
            Some(&["target", "node_modules"][..])
        );
        assert!(!disc.enumerate);
    }
}
