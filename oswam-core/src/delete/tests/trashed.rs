use super::*;

fn bin_fs() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.rooted_at(HOME);
    fs.dir(
        "/Users/tester/.Trash",
        &["/Users/tester/.Trash/a", "/Users/tester/.Trash/b"],
    );
    fs.file("/Users/tester/.Trash/a", 4)
        .inode("/Users/tester/.Trash/a", 7);
    fs.file("/Users/tester/.Trash/b", 6)
        .inode("/Users/tester/.Trash/b", 9);
    fs.file("/Users/tester/keep", 2)
        .inode("/Users/tester/keep", 11);
    fs
}

fn record(path: &str, ino: u64) -> Trashed {
    Trashed {
        path: PathBuf::from(path),
        bytes: 1,
        ino,
        dev: 0,
    }
}

fn empty(fs: &FakeFs, items: &[Trashed]) -> DeleteReport {
    let config = Config::default();
    purge_trashed(&guard(fs, &config), items)
}

#[test]
fn trashing_remembers_where_each_item_went_and_its_identity() {
    let mut fs = tree();
    fs.rooted_at(HOME);
    fs.inode("/Users/tester/cache/a", 7)
        .inode("/Users/tester/cache/b", 9);
    let report = run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Trash,
        false,
    )
    .unwrap();
    let mut got: Vec<(PathBuf, u64, u64)> = report
        .trashed
        .iter()
        .map(|t| (t.path.clone(), t.ino, t.bytes))
        .collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            (PathBuf::from("/Users/tester/.Trash/a"), 7, 4 * 512),
            (PathBuf::from("/Users/tester/.Trash/b"), 9, 6 * 512),
        ]
    );
}

#[test]
fn a_dry_run_or_permanent_delete_puts_nothing_in_the_trash_list() {
    let fs = tree();
    for (disposition, dry_run) in [(Disposition::Trash, true), (Disposition::Permanent, false)] {
        let report = run(
            &fs,
            "/Users/tester/cache",
            &[],
            CleanupKind::DeleteContents,
            disposition,
            dry_run,
        )
        .unwrap();
        assert!(report.trashed.is_empty());
    }
}

#[test]
fn emptying_removes_exactly_what_oswam_put_there() {
    let fs = bin_fs();
    let report = empty(&fs, &[record("/Users/tester/.Trash/a", 7)]);
    assert_eq!(
        *fs.removed.borrow(),
        vec![PathBuf::from("/Users/tester/.Trash/a")]
    );
    assert_eq!(report.bytes(Disposition::Permanent), 4 * 512);
}

#[test]
fn a_same_named_stranger_in_the_trash_is_left_alone() {
    let fs = bin_fs();
    let report = empty(&fs, &[record("/Users/tester/.Trash/b", 8)]);
    assert!(fs.removed.borrow().is_empty());
    assert!(report.done[0].untouched);
}

#[test]
fn nothing_outside_the_trash_is_ever_emptied() {
    let fs = bin_fs();
    let report = empty(
        &fs,
        &[
            record("/Users/tester/keep", 11),
            record("/Users/tester/.Trash", 0),
        ],
    );
    assert!(fs.removed.borrow().is_empty());
    assert!(report.done.iter().all(|d| d.untouched));
}

#[test]
fn an_item_already_gone_from_the_trash_is_reported_not_failed() {
    let fs = bin_fs();
    let report = empty(&fs, &[record("/Users/tester/.Trash/restored", 3)]);
    assert!(report.failed.is_empty());
    assert!(report.done[0].untouched);
}

fn aliased_bin_fs() -> FakeFs {
    let real = "/Volumes/Home/tester";
    let mut fs = FakeFs::default();
    fs.rooted_at(real).home_aliased_as(HOME);
    fs.links.insert(PathBuf::from(HOME), PathBuf::from(real));
    fs.dir(real, &["/Volumes/Home/tester/.Trash"]);
    fs.dir(
        "/Volumes/Home/tester/.Trash",
        &["/Volumes/Home/tester/.Trash/a"],
    );
    fs.file("/Volumes/Home/tester/.Trash/a", 4)
        .inode("/Volumes/Home/tester/.Trash/a", 7);
    fs.file("/Volumes/Home/tester/a", 4)
        .inode("/Volumes/Home/tester/a", 7);
    fs
}

#[test]
fn an_aliased_home_still_empties_what_oswam_trashed() {
    let fs = aliased_bin_fs();
    empty(&fs, &[record("/Volumes/Home/tester/.Trash/a", 7)]);
    assert_eq!(
        *fs.removed.borrow(),
        vec![PathBuf::from("/Volumes/Home/tester/.Trash/a")]
    );
}

#[test]
fn an_aliased_home_never_empties_outside_its_trash() {
    let fs = aliased_bin_fs();
    let report = empty(&fs, &[record("/Volumes/Home/tester/a", 7)]);
    assert!(fs.removed.borrow().is_empty());
    assert!(report.done[0].untouched);
}
