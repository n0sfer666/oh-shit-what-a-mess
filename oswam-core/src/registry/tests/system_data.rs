use super::*;
use crate::risk::RiskLevel;

#[test]
fn generic_macos_junk_lives_in_system_data() {
    for path in [
        "~/Library/Caches",
        "~/Library/Logs",
        "~/Library/Application Support/Caches",
        "~/Library/Saved Application State",
    ] {
        assert!(has_path(SYSTEM_DATA_ID, path), "{path}");
        assert!(!target(SYSTEM_DATA_ID, path).needs_root, "{path}");
    }
}

#[test]
fn sudo_only_targets_moved_into_system_data_still_require_root() {
    for path in [
        "/Library/Developer/CoreSimulator/Caches/dyld",
        "/Library/Logs",
        "/Library/Developer/CommandLineTools",
        "/Library/Application Support/CrashReporter",
        "/System/Volumes/Data/.Spotlight-V100",
        "/System/Volumes/Data/.DocumentRevisions-V100",
    ] {
        assert!(target(SYSTEM_DATA_ID, path).needs_root, "{path}");
    }
    assert!(clean_of(SYSTEM_DATA_ID, &["log", "erase"]).contains(&"--all".to_string()));
}

#[test]
fn macos_owned_space_is_shown_but_never_deletable() {
    for path in [
        "/System/Volumes/VM",
        "/private/var/vm",
        "/private/var/folders",
        "/System/Volumes/Data/.Spotlight-V100",
        "/System/Volumes/Data/.DocumentRevisions-V100",
    ] {
        let t = target(SYSTEM_DATA_ID, path);
        assert_eq!(t.kind, CleanupKind::InfoOnly, "{path}");
        assert_eq!(t.risk, RiskLevel::Danger, "{path}");
    }
}

#[test]
fn the_trash_is_its_own_category_and_not_system_data() {
    assert!(has_path("trash", "~/.Trash"));
    assert!(!has_path(SYSTEM_DATA_ID, "~/.Trash"));
    assert_eq!(target("trash", "~/.Trash").risk, RiskLevel::Danger);
}

#[test]
fn newly_found_space_is_registered_with_an_honest_risk() {
    let downloads = target("simulators", "~/Library/Android/sdk/.downloadIntermediates");
    assert_eq!(downloads.kind, CleanupKind::DeleteContents);
    assert_eq!(downloads.risk, RiskLevel::Caution);
    let html = target(
        "big-data",
        "~/Library/Application Support/Steam/config/htmlcache",
    );
    assert_eq!(html.risk, RiskLevel::Caution);
    assert_eq!(html.blocking_app, Some("steam_osx"));
}

#[test]
fn brew_cleanup_never_autoremoves_formulae_the_user_relies_on() {
    assert_eq!(
        clean_of("dev", &["env", "HOMEBREW_NO_AUTOREMOVE=1"]),
        vec!["env", "HOMEBREW_NO_AUTOREMOVE=1", "brew", "cleanup"]
    );
}
