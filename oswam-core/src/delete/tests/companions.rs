use super::*;

fn avd() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/avd/Pixel.avd", &[]);
    fs.file("/Users/tester/avd/Pixel.ini", 1);
    fs
}

#[test]
fn companions_go_with_the_primary() {
    let fs = avd();
    let ini = vec![PathBuf::from("/Users/tester/avd/Pixel.ini")];
    run(
        &fs,
        "/Users/tester/avd/Pixel.avd",
        &ini,
        CleanupKind::DeletePath,
        Disposition::Trash,
        false,
    )
    .unwrap();
    let trashed = fs.trashed.borrow();
    assert!(trashed.contains(&PathBuf::from("/Users/tester/avd/Pixel.avd")));
    assert!(trashed.contains(&PathBuf::from("/Users/tester/avd/Pixel.ini")));
}

#[test]
fn companions_are_spared_when_primary_is_not_deletable() {
    let fs = avd();
    let ini = vec![PathBuf::from("/Users/tester/avd/Pixel.ini")];
    let report = run(
        &fs,
        "/Users/tester/avd/Pixel.avd",
        &ini,
        CleanupKind::InfoOnly,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert!(report.done.is_empty());
    assert!(fs.trashed.borrow().is_empty());
}

#[test]
fn companions_are_spared_when_primary_deletion_fails() {
    let mut fs = avd();
    fs.deny("/Users/tester/avd/Pixel.avd");
    let ini = vec![PathBuf::from("/Users/tester/avd/Pixel.ini")];
    let report = run(
        &fs,
        "/Users/tester/avd/Pixel.avd",
        &ini,
        CleanupKind::DeletePath,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert!(report.done.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert!(fs.trashed.borrow().is_empty());
}

#[test]
fn a_protected_companion_spares_the_whole_unit() {
    let fs = avd();
    let config = Config {
        protected_paths: vec!["/Users/tester/avd/Pixel.ini".to_string()],
        ..Config::default()
    };
    let ini = vec![PathBuf::from("/Users/tester/avd/Pixel.ini")];
    let report = Deleter::new(Disposition::Trash, false)
        .remove(
            &guard(&fs, &config),
            Path::new("/Users/tester/avd/Pixel.avd"),
            &ini,
            CleanupKind::DeletePath,
        )
        .unwrap();
    assert_eq!(report.done.len(), 1);
    assert!(report.done[0].untouched);
    assert!(fs.trashed.borrow().is_empty());
    assert!(fs.removed.borrow().is_empty());
}

#[test]
fn items_inside_trash_are_removed_permanently() {
    let mut fs = FakeFs::default();
    fs.rooted_at("/Users/tester");
    fs.dir("/Users/tester/.Trash", &["/Users/tester/.Trash/old"]);
    fs.file("/Users/tester/.Trash/old", 4);
    let report = run(
        &fs,
        "/Users/tester/.Trash",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert_eq!(report.done[0].disposition, Disposition::Permanent);
    assert!(fs.trashed.borrow().is_empty());
    assert!(fs
        .removed
        .borrow()
        .contains(&PathBuf::from("/Users/tester/.Trash/old")));
}

#[test]
fn only_the_filesystem_forces_permanent_removal_never_the_risk() {
    let mut fs = FakeFs::default();
    fs.rooted_at("/Users/tester");
    fs.file("/Users/tester/Documents/save.dat", 4);
    let report = run(
        &fs,
        "/Users/tester/Documents/save.dat",
        &[],
        CleanupKind::DeletePath,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert_eq!(report.done[0].disposition, Disposition::Trash);
    assert!(fs
        .trashed
        .borrow()
        .contains(&PathBuf::from("/Users/tester/Documents/save.dat")));
}

#[test]
fn paths_outside_home_cannot_be_trashed() {
    let fs = crate::fsops::RealFs::new().unwrap();
    assert!(!fs.can_trash(Path::new("/Library/Caches/x")));
    assert_eq!(
        effective(&fs, Path::new("/Library/Caches/x"), Disposition::Trash),
        Disposition::Permanent
    );
}

#[test]
fn symlink_not_followed_on_remove() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/d", &["/Users/tester/d/link"]);
    fs.entries.insert(
        PathBuf::from("/Users/tester/d/link"),
        crate::fsops::Meta {
            is_symlink: true,
            uid: 501,
            blocks: 0,
            ..crate::fsops::Meta::default()
        },
    );
    fs.children.insert(
        PathBuf::from("/Users/tester/d/link"),
        vec![PathBuf::from("/Users/tester/d/link/x")],
    );
    run(
        &fs,
        "/Users/tester/d",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Permanent,
        false,
    )
    .unwrap();
    let removed = fs.removed.borrow();
    assert!(removed.contains(&PathBuf::from("/Users/tester/d/link")));
    assert!(!removed.contains(&PathBuf::from("/Users/tester/d/link/x")));
}
