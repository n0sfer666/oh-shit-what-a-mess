use super::*;
use crate::category::CleanupKind;
use crate::discover::matches_any;
use crate::risk::RiskLevel;

fn covers(outer: &Target, inner: &str) -> bool {
    let root = outer.path.trim_end_matches('/');
    let Some(rel) = inner.strip_prefix(&format!("{root}/")) else {
        return false;
    };
    match &outer.discover {
        Some(names) => rel
            .split('/')
            .take(outer.depth + 1)
            .any(|part| matches_any(part, names)),
        None => true,
    }
}

#[test]
fn no_target_is_nested_in_another() {
    let targets: Vec<Target> = all_targets()
        .into_iter()
        .filter(|t| deletes(t) || t.measured)
        .collect();
    for outer in &targets {
        for inner in &targets {
            let path = inner.path.trim_end_matches('/');
            assert!(
                !covers(outer, path),
                "{path} is already covered by {}",
                outer.path
            );
        }
    }
}

#[test]
fn a_discovered_root_covers_only_its_named_folders() {
    let caches = Target::discovered(
        "~/a",
        vec!["Cache"],
        1,
        CleanupKind::DeletePath,
        RiskLevel::Caution,
    );
    assert!(covers(&caches, "~/a/Cache"));
    assert!(covers(&caches, "~/a/x/Cache/y"));
    assert!(!covers(&caches, "~/a/claude-code"));
    assert!(!covers(&caches, "~/ab/Cache"));
    assert!(!covers(&caches, "~/a/x/y/Cache"));
}

#[test]
fn a_top_level_discovery_ignores_nested_namesakes() {
    let caches = Target::discovered(
        "~/a",
        vec!["Cache"],
        0,
        CleanupKind::DeletePath,
        RiskLevel::Caution,
    );
    assert!(covers(&caches, "~/a/Cache"));
    assert!(!covers(&caches, "~/a/x/Cache"));
}

#[test]
fn a_wildcard_name_covers_its_prefixed_folders() {
    let builds = Target::discovered(
        "~/a",
        vec!["cmake-build-*"],
        2,
        CleanupKind::DeletePath,
        RiskLevel::Caution,
    );
    assert!(covers(&builds, "~/a/p/cmake-build-debug"));
    assert!(!covers(&builds, "~/a/p/cmake"));
}
