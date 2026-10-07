use super::*;
use crate::category::CleanupKind;
use crate::risk::RiskLevel;

const ROOT: &str = "~/Library/Application Support/Claude";

#[test]
fn every_claude_target_waits_for_the_app_to_close() {
    let claude = cat("claude");
    assert_eq!(claude.targets.len(), 4);
    for t in &claude.targets {
        assert_eq!(t.blocking_app, Some("Claude"), "{}", t.path);
        assert_eq!(t.risk, RiskLevel::Caution, "{}", t.path);
        assert_eq!(t.kind, CleanupKind::DeletePath, "{}", t.path);
    }
}

#[test]
fn claude_caches_are_the_browser_cache_folders_at_the_top_level() {
    let caches = target("claude", ROOT);
    assert_eq!(caches.depth, 0);
    assert_eq!(
        caches.discover.as_deref(),
        Some(&super::super::browsers::CACHE_DIRS[..])
    );
}

#[test]
fn bundled_claude_code_builds_are_listed_one_by_one() {
    for dir in ["claude-code", "claude-code-vm"] {
        let builds = target("claude", &format!("{ROOT}/{dir}"));
        assert!(builds.enumerate && builds.versioned, "{dir}");
    }
}

#[test]
fn the_vm_base_image_leaves_together_with_its_origin_manifest() {
    let rootfs = target(
        "claude",
        &format!("{ROOT}/vm_bundles/claudevm.bundle/rootfs.img"),
    );
    assert_eq!(rootfs.companions, vec![".rootfs.img.origin"]);
}

#[test]
fn cowork_sessions_and_the_bundle_itself_are_never_targets() {
    for t in all_targets() {
        assert!(!t.path.contains("sessiondata.img"), "{}", t.path);
        assert!(!t.path.ends_with("claudevm.bundle"), "{}", t.path);
        assert!(!t.path.ends_with("vm_bundles"), "{}", t.path);
    }
}

#[test]
fn claude_is_not_shown_twice_as_a_parent_and_its_children() {
    assert!(!has_path("big-data", ROOT));
}
