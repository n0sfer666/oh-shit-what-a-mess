use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Caution,
    Danger,
    Never,
}

#[derive(Debug, Clone, Default)]
pub struct PathFacts {
    pub path: PathBuf,
    pub is_root_owned: bool,
    pub is_sip_protected: bool,
    pub is_dataless: bool,
    pub user_protected: bool,
    pub process_holding: bool,
}

impl PathFacts {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }
}

pub fn classify(facts: &PathFacts, default_risk: RiskLevel) -> RiskLevel {
    if is_never(facts) || facts.user_protected {
        return RiskLevel::Never;
    }
    let mut level = default_risk;
    if facts.process_holding && level < RiskLevel::Caution {
        level = RiskLevel::Caution;
    }
    level
}

fn is_never(facts: &PathFacts) -> bool {
    facts.is_sip_protected
        || facts.is_root_owned
        || facts.is_dataless
        || is_photoslibrary_internal(&facts.path)
}

fn is_photoslibrary_internal(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| s.ends_with(".photoslibrary"))
    }) && path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| !n.ends_with(".photoslibrary"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn never_facts() -> Vec<PathFacts> {
        let mut sip = PathFacts::new("/System/Library/CoreServices/SystemVersion.plist");
        sip.is_sip_protected = true;
        let mut root = PathFacts::new("/Library/Preferences/com.apple.x.plist");
        root.is_root_owned = true;
        let mut icloud =
            PathFacts::new("/Users/n0sfer/Library/Mobile Documents/com~apple~CloudDocs/d.pages");
        icloud.is_dataless = true;
        let photos = PathFacts::new("/Users/n0sfer/Pictures/P.photoslibrary/originals/0/IMG.heic");
        vec![sip, root, icloud, photos]
    }

    #[test]
    fn never_signals_override_default() {
        for f in never_facts() {
            assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Never);
        }
    }

    #[test]
    fn user_protected_is_never() {
        let mut f = PathFacts::new("/Users/n0sfer/Library/Caches/important");
        f.user_protected = true;
        assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Never);
    }

    #[test]
    fn default_risk_passthrough() {
        let f = PathFacts::new("/Users/n0sfer/Library/Caches/yarn");
        assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Safe);
        assert_eq!(classify(&f, RiskLevel::Caution), RiskLevel::Caution);
    }

    #[test]
    fn process_holding_bumps_safe_to_caution() {
        let mut f = PathFacts::new("/Users/n0sfer/Library/Caches/Arc");
        f.process_holding = true;
        assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Caution);
    }

    #[test]
    fn process_holding_does_not_lower_danger() {
        let mut f = PathFacts::new("/Users/n0sfer/Library/Caches/Arc");
        f.process_holding = true;
        assert_eq!(classify(&f, RiskLevel::Danger), RiskLevel::Danger);
    }

    #[test]
    fn risk_level_ordering() {
        assert!(RiskLevel::Safe < RiskLevel::Caution);
        assert!(RiskLevel::Caution < RiskLevel::Danger);
        assert!(RiskLevel::Danger < RiskLevel::Never);
    }
}
