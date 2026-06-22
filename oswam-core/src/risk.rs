use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Caution,
    Danger,
    Never,
}

#[derive(Debug, Clone)]
pub struct PathFacts {
    pub path: PathBuf,
    pub is_root_owned: bool,
    pub is_sip_protected: bool,
    pub is_dataless: bool,
}

pub fn classify(facts: &PathFacts) -> RiskLevel {
    if is_never(facts) {
        return RiskLevel::Never;
    }
    RiskLevel::Safe
}

fn is_never(facts: &PathFacts) -> bool {
    facts.is_sip_protected
        || facts.is_root_owned
        || facts.is_dataless
        || is_photoslibrary_internal(&facts.path)
}

fn is_photoslibrary_internal(path: &std::path::Path) -> bool {
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

    fn facts(path: &str) -> PathFacts {
        PathFacts {
            path: PathBuf::from(path),
            is_root_owned: false,
            is_sip_protected: false,
            is_dataless: false,
        }
    }

    #[test]
    fn sip_protected_is_never() {
        let mut f = facts("/System/Library/CoreServices/SystemVersion.plist");
        f.is_sip_protected = true;
        assert_eq!(classify(&f), RiskLevel::Never);
    }

    #[test]
    fn root_owned_is_never() {
        let mut f = facts("/Library/Preferences/com.apple.something.plist");
        f.is_root_owned = true;
        assert_eq!(classify(&f), RiskLevel::Never);
    }

    #[test]
    fn icloud_dataless_stub_is_never() {
        let mut f = facts("/Users/n0sfer/Library/Mobile Documents/com~apple~CloudDocs/doc.pages");
        f.is_dataless = true;
        assert_eq!(classify(&f), RiskLevel::Never);
    }

    #[test]
    fn photoslibrary_internals_is_never() {
        let f = facts("/Users/n0sfer/Pictures/Photos Library.photoslibrary/originals/0/IMG.heic");
        assert_eq!(classify(&f), RiskLevel::Never);
    }
}
