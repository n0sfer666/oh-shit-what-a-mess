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
    if facts.process_holding {
        return default_risk.max(RiskLevel::Caution);
    }
    default_risk
}

fn is_never(facts: &PathFacts) -> bool {
    facts.is_sip_protected
        || facts.is_root_owned
        || facts.is_dataless
        || is_photoslibrary_internal(&facts.path)
        || is_never_path(&facts.path)
}

fn is_never_path(path: &Path) -> bool {
    const NEVER_ANYWHERE: [&str; 1] = [".Spotlight-V100"];
    const NEVER_UNDER_LIBRARY: [&str; 3] = ["Keychains", "Containers", "Mobile Documents"];
    const NEVER_PREFIXES: [&str; 7] = [
        "/System",
        "/private/var/db",
        "/private/var/folders",
        "/private/var/vm",
        "/var/db",
        "/var/folders",
        "/var/vm",
    ];
    let names: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let anywhere = names.iter().any(|s| NEVER_ANYWHERE.contains(s));
    let under_library = names
        .windows(2)
        .any(|pair| pair[0] == "Library" && NEVER_UNDER_LIBRARY.contains(&pair[1]));
    anywhere || under_library || NEVER_PREFIXES.iter().any(|p| path.starts_with(p))
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
            PathFacts::new("/Users/tester/Library/Mobile Documents/com~apple~CloudDocs/d.pages");
        icloud.is_dataless = true;
        let photos = PathFacts::new("/Users/tester/Pictures/P.photoslibrary/originals/0/IMG.heic");
        vec![sip, root, icloud, photos]
    }

    #[test]
    fn never_signals_override_default() {
        for f in never_facts() {
            assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Never);
        }
    }

    #[test]
    fn system_paths_are_never() {
        for p in [
            "/private/var/folders/xy/abc/T/cache",
            "/private/var/db/spotlight",
            "/private/var/vm/sleepimage",
            "/var/folders/xy/abc/T/cache",
            "/var/db/spotlight",
            "/Users/tester/Library/Keychains/login.keychain-db",
            "/Users/tester/Library/Containers/com.app/Data/x",
            "/Volumes/Data/.Spotlight-V100/Store",
            "/System/Library/Caches/x",
        ] {
            assert_eq!(
                classify(&PathFacts::new(p), RiskLevel::Safe),
                RiskLevel::Never,
                "{p}"
            );
        }
    }

    #[test]
    fn ordinary_cache_paths_stay_deletable() {
        for p in [
            "/Users/tester/Library/Caches/go-build",
            "/Users/tester/Library/Application Support/Google/Chrome/Default/Cache",
            "/Users/tester/projects/app/node_modules",
            "/Users/tester/Library/Developer/CoreSimulator/Devices/A/data/Containers/Bundle",
            "/Users/tester/projects/app/Containers/state",
        ] {
            assert_ne!(
                classify(&PathFacts::new(p), RiskLevel::Safe),
                RiskLevel::Never,
                "{p}"
            );
        }
    }

    #[test]
    fn user_protected_is_never() {
        let mut f = PathFacts::new("/Users/tester/Library/Caches/important");
        f.user_protected = true;
        assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Never);
    }

    #[test]
    fn default_risk_passthrough() {
        let f = PathFacts::new("/Users/tester/Library/Caches/yarn");
        assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Safe);
        assert_eq!(classify(&f, RiskLevel::Caution), RiskLevel::Caution);
    }

    #[test]
    fn process_holding_bumps_safe_to_caution() {
        let mut f = PathFacts::new("/Users/tester/Library/Caches/Arc");
        f.process_holding = true;
        assert_eq!(classify(&f, RiskLevel::Safe), RiskLevel::Caution);
    }

    #[test]
    fn a_busy_build_cache_is_not_called_user_data() {
        let mut f = PathFacts::new("/Users/tester/_dev/app/target");
        f.process_holding = true;
        assert_eq!(classify(&f, RiskLevel::Caution), RiskLevel::Caution);
    }

    #[test]
    fn process_holding_does_not_lower_danger() {
        let mut f = PathFacts::new("/Users/tester/Library/Caches/Arc");
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
