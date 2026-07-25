use super::*;
use crate::config::Config;
use crate::fsops::fake::FakeFs;

mod companions;
mod guarded;
mod hardlinks;
mod partial;
mod untouched;

const HOME: &str = "/Users/tester";

fn guard<'a>(fs: &'a FakeFs, config: &'a Config) -> Guard<'a, FakeFs> {
    Guard::new(fs, config, Path::new(HOME))
}

fn run(
    fs: &FakeFs,
    path: &str,
    companions: &[PathBuf],
    kind: CleanupKind,
    disposition: Disposition,
    dry_run: bool,
) -> io::Result<DeleteReport> {
    let config = Config::default();
    Deleter::new(disposition, dry_run).remove(
        &guard(fs, &config),
        Path::new(path),
        companions,
        kind,
    )
}

fn purge(fs: &FakeFs, config: &Config, path: &str, kind: CleanupKind) -> DeleteReport {
    Deleter::new(Disposition::Permanent, false)
        .remove(&guard(fs, config), Path::new(path), &[], kind)
        .unwrap()
}

fn tree() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/cache",
        &["/Users/tester/cache/a", "/Users/tester/cache/b"],
    );
    fs.file("/Users/tester/cache/a", 4);
    fs.file("/Users/tester/cache/b", 6);
    fs
}

#[test]
fn dry_run_touches_nothing_but_reports_sizes() {
    let fs = tree();
    let report = run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Permanent,
        true,
    )
    .unwrap();
    assert_eq!(report.done.len(), 2);
    assert!(fs.removed.borrow().is_empty());
    assert!(fs.trashed.borrow().is_empty());
    assert_eq!(report.bytes(Disposition::Permanent), (4 + 6) * 512);
}

#[test]
fn permanent_removes_children() {
    let fs = tree();
    run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Permanent,
        false,
    )
    .unwrap();
    let removed = fs.removed.borrow();
    assert!(removed.contains(&PathBuf::from("/Users/tester/cache/a")));
    assert!(removed.contains(&PathBuf::from("/Users/tester/cache/b")));
    assert!(!removed.contains(&PathBuf::from("/Users/tester/cache")));
}

#[test]
fn trash_moves_children() {
    let fs = tree();
    run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert_eq!(fs.trashed.borrow().len(), 2);
    assert!(fs.removed.borrow().is_empty());
}

#[test]
fn info_only_yields_nothing() {
    let fs = tree();
    let report = run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::InfoOnly,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert!(report.done.is_empty());
}

#[test]
fn unreadable_directory_is_an_error_not_a_silent_zero() {
    let fs = FakeFs::default();
    let err = run(
        &fs,
        "/Users/tester/denied",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Trash,
        false,
    );
    assert!(err.is_err());
}

#[test]
fn a_plain_file_entry_deletes_itself_not_its_contents() {
    let mut fs = FakeFs::default();
    fs.file("/Users/tester/cache/one.plist", 4);
    let report = run(
        &fs,
        "/Users/tester/cache/one.plist",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert_eq!(report.done.len(), 1);
    assert!(fs
        .trashed
        .borrow()
        .contains(&PathBuf::from("/Users/tester/cache/one.plist")));
}

#[test]
fn a_failed_removal_is_reported_not_swallowed() {
    let mut fs = tree();
    fs.deny("/Users/tester/cache/a");
    let report = run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Trash,
        false,
    )
    .unwrap();
    assert_eq!(report.done.len(), 1);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(
        report.failed[0].path,
        PathBuf::from("/Users/tester/cache/a")
    );
}
