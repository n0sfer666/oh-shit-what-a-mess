use super::*;

fn nested() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/cache",
        &["/Users/tester/cache/a", "/Users/tester/cache/keep"],
    );
    fs.file("/Users/tester/cache/a", 4);
    fs.dir(
        "/Users/tester/cache/keep",
        &["/Users/tester/cache/keep/inner"],
    );
    fs.file("/Users/tester/cache/keep/inner", 6);
    fs
}

#[test]
fn a_protected_subtree_survives_deletion_of_its_neighbours() {
    let fs = nested();
    let config = Config {
        protected_paths: vec!["~/cache/keep".into()],
        ..Config::default()
    };
    purge(
        &fs,
        &config,
        "/Users/tester/cache",
        CleanupKind::DeleteContents,
    );
    let removed = fs.removed.borrow();
    assert!(removed.contains(&PathBuf::from("/Users/tester/cache/a")));
    assert!(!removed.contains(&PathBuf::from("/Users/tester/cache/keep")));
    assert!(!removed.contains(&PathBuf::from("/Users/tester/cache/keep/inner")));
}

#[test]
fn recursion_stops_at_a_protected_descendant() {
    let fs = nested();
    let config = Config {
        protected_paths: vec!["~/cache/keep/inner".into()],
        ..Config::default()
    };
    let report = purge(&fs, &config, "/Users/tester/cache", CleanupKind::DeletePath);
    assert!(fs.removed.borrow().is_empty());
    assert_eq!(report.done.len(), 1);
    assert!(report.done[0].untouched);
}

#[test]
fn an_ignored_child_keeps_its_parent_directory_alive() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/d",
        &["/Users/tester/d/x", "/Users/tester/d/y.lock"],
    );
    fs.file("/Users/tester/d/x", 4);
    fs.file("/Users/tester/d/y.lock", 2);
    let config = Config {
        ignore_globs: vec!["**/*.lock".into()],
        ..Config::default()
    };
    let report = purge(&fs, &config, "/Users/tester/d", CleanupKind::DeletePath);
    let removed = fs.removed.borrow();
    assert!(removed.contains(&PathBuf::from("/Users/tester/d/x")));
    assert!(!removed.contains(&PathBuf::from("/Users/tester/d")));
    assert_eq!(report.done[0].physical_bytes, 4 * 512);
}
