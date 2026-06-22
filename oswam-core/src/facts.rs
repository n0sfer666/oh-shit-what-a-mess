use crate::fsops::{is_dataless, is_root_owned, is_sip_protected, Meta};
use crate::risk::PathFacts;
use std::path::Path;

pub fn facts_from_meta(
    path: &Path,
    meta: &Meta,
    user_protected: bool,
    process_holding: bool,
) -> PathFacts {
    PathFacts {
        path: path.to_path_buf(),
        is_root_owned: is_root_owned(meta.uid),
        is_sip_protected: is_sip_protected(meta.flags),
        is_dataless: is_dataless(meta.flags),
        user_protected,
        process_holding,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsops::{SF_DATALESS, SF_RESTRICTED};

    #[test]
    fn maps_meta_flags_into_facts() {
        let meta = Meta {
            uid: 0,
            flags: SF_RESTRICTED | SF_DATALESS,
            ..Meta::default()
        };
        let f = facts_from_meta(Path::new("/x"), &meta, true, true);
        assert!(f.is_root_owned);
        assert!(f.is_sip_protected);
        assert!(f.is_dataless);
        assert!(f.user_protected);
        assert!(f.process_holding);
    }

    #[test]
    fn user_meta_is_clean() {
        let meta = Meta {
            uid: 501,
            flags: 0,
            ..Meta::default()
        };
        let f = facts_from_meta(Path::new("/x"), &meta, false, false);
        assert!(!f.is_root_owned);
        assert!(!f.is_sip_protected);
        assert!(!f.is_dataless);
    }
}
