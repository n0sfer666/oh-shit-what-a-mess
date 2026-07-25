use super::*;

#[test]
fn a_dry_run_counts_hardlinked_bytes_once() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/cache",
        &["/Users/tester/cache/x", "/Users/tester/cache/y"],
    );
    fs.hardlink("/Users/tester/cache/x", 100, 7);
    fs.hardlink("/Users/tester/cache/y", 100, 7);
    let report = run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Permanent,
        true,
    )
    .unwrap();
    assert_eq!(report.bytes(Disposition::Permanent), 100 * 512);
}

#[test]
fn a_real_purge_counts_hardlinked_bytes_once_even_as_links_disappear() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/cache",
        &["/Users/tester/cache/x", "/Users/tester/cache/y"],
    );
    fs.hardlink("/Users/tester/cache/x", 100, 7);
    fs.hardlink("/Users/tester/cache/y", 100, 7);
    let report = run(
        &fs,
        "/Users/tester/cache",
        &[],
        CleanupKind::DeleteContents,
        Disposition::Permanent,
        false,
    )
    .unwrap();
    assert_eq!(fs.removed.borrow().len(), 2);
    assert_eq!(report.bytes(Disposition::Permanent), 100 * 512);
}
