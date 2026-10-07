use super::super::*;
use super::{one, Env};
use crate::category::Target;
use crate::fsops::fake::FakeFs;
use crate::select::is_deletable;

const BUNDLE: &str = "/Users/tester/Library/Application Support/Claude/vm_bundles/claudevm.bundle";

fn rootfs_fs(with_origin: bool) -> FakeFs {
    let mut fs = FakeFs::default();
    let rootfs = format!("{BUNDLE}/rootfs.img");
    let origin = format!("{BUNDLE}/.rootfs.img.origin");
    let session = format!("{BUNDLE}/sessiondata.img");
    let mut children = vec![rootfs.as_str(), session.as_str()];
    if with_origin {
        children.push(origin.as_str());
        fs.file(&origin, 1);
    }
    fs.dir(BUNDLE, &children);
    fs.file(&rootfs, 100);
    fs.file(&session, 50);
    fs
}

fn rootfs_cats() -> Vec<Category> {
    one(
        "claude",
        Target::new(
            "~/Library/Application Support/Claude/vm_bundles/claudevm.bundle/rootfs.img",
            CleanupKind::DeletePath,
            RiskLevel::Caution,
        )
        .with_companions(&[".rootfs.img.origin"])
        .blocked_while_running("Claude"),
    )
}

#[test]
fn a_target_is_blocked_while_its_app_runs() {
    let result = Env::running(rootfs_fs(true), &["Claude"]).scan(&rootfs_cats());
    let entry = &result.categories[0].entries[0];
    assert!(entry.blocked);
    assert_eq!(entry.blocking_app.as_deref(), Some("Claude"));
    assert!(!is_deletable(entry));
    assert_eq!(result.total_bytes, 0);
    assert_eq!(entry.physical_bytes, (100 + 1) * 512);
}

#[test]
fn a_target_is_free_once_its_app_is_closed() {
    let result = Env::running(rootfs_fs(true), &["Claude Helper"]).scan(&rootfs_cats());
    let entry = &result.categories[0].entries[0];
    assert!(!entry.blocked);
    assert_eq!(entry.blocking_app.as_deref(), Some("Claude"));
    assert!(is_deletable(entry));
    assert_eq!(result.total_bytes, (100 + 1) * 512);
}

#[test]
fn the_image_carries_its_manifest_and_leaves_the_session_disk() {
    let entries = &Env::new(rootfs_fs(true)).scan(&rootfs_cats()).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].companions,
        vec![PathBuf::from(format!("{BUNDLE}/.rootfs.img.origin"))]
    );
}

#[test]
fn a_missing_manifest_is_not_promised() {
    let entries = &Env::new(rootfs_fs(false)).scan(&rootfs_cats()).categories[0].entries;
    assert!(entries[0].companions.is_empty());
    assert_eq!(entries[0].physical_bytes, 100 * 512);
}

#[test]
fn a_protected_manifest_keeps_the_image_for_reference_only() {
    let env = Env::protecting(rootfs_fs(true), &format!("{BUNDLE}/.rootfs.img.origin"));
    let entry = &env.scan(&rootfs_cats()).categories[0].entries[0];
    assert_eq!(entry.kind, CleanupKind::InfoOnly);
    assert!(!is_deletable(entry));
}

#[test]
fn a_view_only_image_is_not_promised_after_closing_the_app() {
    let mut env = Env::protecting(rootfs_fs(true), &format!("{BUNDLE}/.rootfs.img.origin"));
    env.probe = crate::process::LsofProbe::from_paths(vec![]).with_running(&["Claude"]);
    let entry = &env.scan(&rootfs_cats()).categories[0].entries[0];
    assert_eq!(entry.kind, CleanupKind::InfoOnly);
    assert!(!entry.blocked);
}

#[test]
fn a_never_entry_is_not_promised_after_closing_the_app() {
    let mut env = Env::protecting(rootfs_fs(true), BUNDLE);
    env.probe = crate::process::LsofProbe::from_paths(vec![]).with_running(&["Claude"]);
    let entry = &env.scan(&rootfs_cats()).categories[0].entries[0];
    assert_eq!(entry.risk, RiskLevel::Never);
    assert!(!entry.blocked);
}

#[test]
fn plain_enumeration_keeps_hidden_files() {
    let mut fs = FakeFs::default();
    let root = "/Users/tester/Library/Caches";
    let hidden = format!("{root}/.cache-db");
    fs.dir(root, &[&hidden]);
    fs.file(&hidden, 1);
    let cats = one(
        "system",
        Target::enumerated("~/Library/Caches", CleanupKind::DeletePath, RiskLevel::Safe),
    );
    let entries = &Env::new(fs).scan(&cats).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display, ".cache-db");
}

#[test]
fn versioned_enumeration_skips_hidden_files_but_keeps_hidden_folders() {
    let mut fs = FakeFs::default();
    let root = "/Users/tester/Library/Application Support/Claude/claude-code-vm";
    let marker = format!("{root}/.sdk-version");
    let build = format!("{root}/2.1.289");
    let hidden_dir = format!("{root}/.staging");
    fs.dir(root, &[&marker, &build, &hidden_dir]);
    fs.file(&marker, 1);
    fs.dir(&build, &[]);
    fs.dir(&hidden_dir, &[]);
    let cats = one(
        "claude",
        Target::enumerated(
            "~/Library/Application Support/Claude/claude-code-vm",
            CleanupKind::DeletePath,
            RiskLevel::Caution,
        )
        .versioned(),
    );
    let names: Vec<String> = Env::new(fs).scan(&cats).categories[0]
        .entries
        .iter()
        .map(|e| e.display.clone())
        .collect();
    assert_eq!(
        names,
        vec!["claude-code-vm/2.1.289", "claude-code-vm/.staging"]
    );
}
