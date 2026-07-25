use super::*;

fn bin(fs: &FakeFs, config: &Config, path: &str, kind: CleanupKind) -> DeleteReport {
    Deleter::new(Disposition::Trash, false)
        .remove(&guard(fs, config), Path::new(path), &[], kind)
        .unwrap()
}

fn locked_grandchild() -> (FakeFs, Config) {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/cache", &["/Users/tester/cache/sub"]);
    fs.dir(
        "/Users/tester/cache/sub",
        &[
            "/Users/tester/cache/sub/x",
            "/Users/tester/cache/sub/y.lock",
        ],
    );
    fs.file("/Users/tester/cache/sub/x", 4);
    fs.file("/Users/tester/cache/sub/y.lock", 2);
    let config = Config {
        ignore_globs: vec!["**/*.lock".into()],
        ..Config::default()
    };
    (fs, config)
}

#[test]
fn an_ignored_grandchild_is_not_swept_into_the_trash() {
    let (fs, config) = locked_grandchild();
    let report = bin(&fs, &config, "/Users/tester/cache", CleanupKind::DeletePath);
    let trashed = fs.trashed.borrow();
    assert!(trashed.contains(&PathBuf::from("/Users/tester/cache/sub/x")));
    assert!(!trashed.contains(&PathBuf::from("/Users/tester/cache")));
    assert!(!trashed.contains(&PathBuf::from("/Users/tester/cache/sub")));
    assert!(!trashed.contains(&PathBuf::from("/Users/tester/cache/sub/y.lock")));
    assert!(report.done[0].partial);
}

#[test]
fn a_clean_subtree_still_goes_to_the_trash_whole() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/cache", &["/Users/tester/cache/sub"]);
    fs.dir("/Users/tester/cache/sub", &["/Users/tester/cache/sub/x"]);
    fs.file("/Users/tester/cache/sub/x", 4);
    let report = bin(
        &fs,
        &Config::default(),
        "/Users/tester/cache",
        CleanupKind::DeletePath,
    );
    assert_eq!(
        *fs.trashed.borrow(),
        vec![PathBuf::from("/Users/tester/cache")]
    );
    assert!(!report.done[0].partial);
}

#[test]
fn a_partial_primary_leaves_its_companions_alone() {
    let (mut fs, config) = locked_grandchild();
    fs.file("/Users/tester/cache.ini", 1);
    let report = Deleter::new(Disposition::Permanent, false)
        .remove(
            &guard(&fs, &config),
            Path::new("/Users/tester/cache"),
            &[PathBuf::from("/Users/tester/cache.ini")],
            CleanupKind::DeletePath,
        )
        .unwrap();
    assert_eq!(report.done.len(), 1);
    assert!(!fs
        .removed
        .borrow()
        .contains(&PathBuf::from("/Users/tester/cache.ini")));
}

#[test]
fn a_dry_run_predicts_the_same_partial_outcome() {
    let (mut fs, config) = locked_grandchild();
    fs.file("/Users/tester/cache.ini", 1);
    let report = Deleter::new(Disposition::Permanent, true)
        .remove(
            &guard(&fs, &config),
            Path::new("/Users/tester/cache"),
            &[PathBuf::from("/Users/tester/cache.ini")],
            CleanupKind::DeletePath,
        )
        .unwrap();
    assert_eq!(report.done.len(), 1);
    assert!(report.done[0].partial);
}

#[test]
fn a_failing_grandchild_does_not_erase_what_was_already_removed() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/cache",
        &["/Users/tester/cache/a", "/Users/tester/cache/sub"],
    );
    fs.file("/Users/tester/cache/a", 4);
    fs.dir("/Users/tester/cache/sub", &["/Users/tester/cache/sub/b"]);
    fs.file("/Users/tester/cache/sub/b", 2);
    fs.deny("/Users/tester/cache/sub/b");
    let report = purge(
        &fs,
        &Config::default(),
        "/Users/tester/cache",
        CleanupKind::DeletePath,
    );
    assert!(fs
        .removed
        .borrow()
        .contains(&PathBuf::from("/Users/tester/cache/a")));
    assert_eq!(report.done.len(), 1);
    assert!(report.done[0].partial);
    assert_eq!(report.done[0].physical_bytes, 4 * 512);
    assert_eq!(
        report.failed.iter().map(|f| &f.path).collect::<Vec<_>>(),
        vec![&PathBuf::from("/Users/tester/cache/sub/b")]
    );
}

#[test]
fn a_partial_trash_promises_only_what_actually_moved() {
    let (fs, config) = locked_grandchild();
    let report = bin(&fs, &config, "/Users/tester/cache", CleanupKind::DeletePath);
    assert_eq!(report.done[0].physical_bytes, 4 * 512);
}

#[test]
fn an_unreadable_child_is_named_not_just_counted_as_kept() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/d",
        &["/Users/tester/d/x", "/Users/tester/d/ghost"],
    );
    fs.file("/Users/tester/d/x", 4);
    let report = purge(
        &fs,
        &Config::default(),
        "/Users/tester/d",
        CleanupKind::DeletePath,
    );
    assert_eq!(
        report.failed.iter().map(|f| &f.path).collect::<Vec<_>>(),
        vec![&PathBuf::from("/Users/tester/d/ghost")]
    );
    assert!(report.done[0].partial);
}

#[test]
fn a_filtered_sibling_does_not_orphan_the_companions() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/avd",
        &["/Users/tester/avd/Pixel.avd", "/Users/tester/avd/keep.lock"],
    );
    fs.dir("/Users/tester/avd/Pixel.avd", &[]);
    fs.file("/Users/tester/avd/keep.lock", 2);
    fs.file("/Users/tester/avd/Pixel.ini", 1);
    let config = Config {
        ignore_globs: vec!["**/*.lock".into()],
        ..Config::default()
    };
    Deleter::new(Disposition::Permanent, false)
        .remove(
            &guard(&fs, &config),
            Path::new("/Users/tester/avd"),
            &[PathBuf::from("/Users/tester/avd/Pixel.ini")],
            CleanupKind::DeleteContents,
        )
        .unwrap();
    let removed = fs.removed.borrow();
    assert!(removed.contains(&PathBuf::from("/Users/tester/avd/Pixel.avd")));
    assert!(removed.contains(&PathBuf::from("/Users/tester/avd/Pixel.ini")));
    assert!(!removed.contains(&PathBuf::from("/Users/tester/avd/keep.lock")));
}
