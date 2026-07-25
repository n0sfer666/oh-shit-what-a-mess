use super::*;

#[test]
fn an_untouched_item_is_reported_instead_of_vanishing() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/d", &["/Users/tester/d/a.lock"]);
    fs.file("/Users/tester/d/a.lock", 2);
    let config = Config {
        ignore_globs: vec!["**/*.lock".into()],
        ..Config::default()
    };
    let report = purge(&fs, &config, "/Users/tester/d", CleanupKind::DeletePath);
    assert!(fs.removed.borrow().is_empty());
    assert_eq!(report.done.len(), 1);
    assert!(report.done[0].untouched);
    assert!(!report.done[0].partial);
    assert_eq!(report.done[0].physical_bytes, 0);
}

#[test]
fn a_guard_blocked_path_is_reported_as_untouched() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/d", &[]);
    let config = Config {
        ignore_globs: vec!["**/d".into()],
        ..Config::default()
    };
    let report = purge(&fs, &config, "/Users/tester/d", CleanupKind::DeletePath);
    assert!(fs.removed.borrow().is_empty());
    assert_eq!(report.done.len(), 1);
    assert!(report.done[0].untouched);
    assert_eq!(report.done[0].physical_bytes, 0);
}

#[test]
fn a_guard_blocked_path_still_spares_its_companions() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/avd/Pixel.avd", &[]);
    fs.file("/Users/tester/avd/Pixel.ini", 1);
    let config = Config {
        ignore_globs: vec!["**/Pixel.avd".into()],
        ..Config::default()
    };
    Deleter::new(Disposition::Permanent, false)
        .remove(
            &guard(&fs, &config),
            Path::new("/Users/tester/avd/Pixel.avd"),
            &[PathBuf::from("/Users/tester/avd/Pixel.ini")],
            CleanupKind::DeletePath,
        )
        .unwrap();
    assert!(fs.removed.borrow().is_empty());
}
