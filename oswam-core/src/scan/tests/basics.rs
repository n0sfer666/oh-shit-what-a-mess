use super::*;
use crate::category::builtin_categories;

#[test]
fn skips_missing_targets() {
    let res = Env::new(FakeFs::default()).scan(&builtin_categories());
    assert!(res.categories.iter().all(|c| c.entries.is_empty()));
    assert_eq!(res.total_bytes, 0);
}

#[test]
fn computes_size_and_risk_for_existing() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/.npm", &["/Users/tester/.npm/x"]);
    fs.file("/Users/tester/.npm/x", 10);
    let cats = one(
        "dev",
        Target::new("~/.npm", CleanupKind::DeleteContents, RiskLevel::Safe),
    );
    let entry = &Env::new(fs).scan(&cats).categories[0].entries[0];
    assert_eq!(entry.physical_bytes, 10 * 512);
    assert_eq!(entry.risk, RiskLevel::Safe);
}

#[test]
fn delete_contents_on_a_plain_file_promises_the_file_itself() {
    let mut fs = FakeFs::default();
    fs.file("/Users/tester/.npm", 10);
    let cats = one(
        "dev",
        Target::new("~/.npm", CleanupKind::DeleteContents, RiskLevel::Safe),
    );
    let entry = &Env::new(fs).scan(&cats).categories[0].entries[0];
    assert_eq!(entry.physical_bytes, 10 * 512);
}

#[test]
fn delete_contents_on_a_symlink_promises_only_the_link() {
    let mut fs = FakeFs::default();
    fs.symlink("/Users/tester/.npm", 1, &["/Users/tester/volume/big"]);
    fs.file("/Users/tester/volume/big", 4096);
    let cats = one(
        "dev",
        Target::new("~/.npm", CleanupKind::DeleteContents, RiskLevel::Safe),
    );
    let entry = &Env::new(fs).scan(&cats).categories[0].entries[0];
    assert_eq!(entry.physical_bytes, 512);
}

#[test]
fn delete_path_counts_the_directory_itself_too() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/.npm", &["/Users/tester/.npm/x"]);
    fs.file("/Users/tester/.npm/x", 10);
    let cats = one(
        "dev",
        Target::new("~/.npm", CleanupKind::DeletePath, RiskLevel::Safe),
    );
    let entry = &Env::new(fs).scan(&cats).categories[0].entries[0];
    assert_eq!(entry.physical_bytes, (8 + 10) * 512);
}

#[test]
fn enumerate_expands_children_into_entries() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/Library/Caches",
        &[
            "/Users/tester/Library/Caches/Homebrew",
            "/Users/tester/Library/Caches/Yarn",
        ],
    );
    fs.dir("/Users/tester/Library/Caches/Homebrew", &[]);
    fs.dir("/Users/tester/Library/Caches/Yarn", &[]);
    let cats = one(
        "system",
        Target::enumerated(
            "~/Library/Caches",
            CleanupKind::DeleteContents,
            RiskLevel::Safe,
        ),
    );
    let entries = &Env::new(fs).scan(&cats).categories[0].entries;
    let displays: Vec<&str> = entries.iter().map(|e| e.display.as_str()).collect();
    assert_eq!(displays.len(), 2);
    assert!(displays.contains(&"Homebrew"));
    assert!(displays.contains(&"Yarn"));
}

#[test]
fn enumerate_through_an_aliased_root_yields_real_paths() {
    let mut fs = FakeFs::default();
    fs.dir("/Volumes/Work/Caches", &["/Volumes/Work/Caches/Yarn"]);
    fs.dir("/Volumes/Work/Caches/Yarn", &[]);
    fs.link("/Users/tester/Library/Caches", "/Volumes/Work/Caches");
    let cats = one(
        "system",
        Target::enumerated(
            "~/Library/Caches",
            CleanupKind::DeleteContents,
            RiskLevel::Safe,
        ),
    );
    let entries = &Env::new(fs).scan(&cats).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, PathBuf::from("/Volumes/Work/Caches/Yarn"));
    assert_eq!(entries[0].display, "Yarn");
}

#[test]
fn process_holding_bumps_risk_in_scan() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/Library/Caches/Arc", &[]);
    let cats = one(
        "browsers",
        Target::new(
            "~/Library/Caches/Arc",
            CleanupKind::DeleteContents,
            RiskLevel::Safe,
        ),
    );
    let env = Env::holding(fs, "/Users/tester/Library/Caches/Arc/db");
    assert_eq!(
        env.scan(&cats).categories[0].entries[0].risk,
        RiskLevel::Caution
    );
}

#[test]
fn progress_reaches_total_targets() {
    let env = Env::new(FakeFs::default());
    let cats = builtin_categories();
    let total_targets: usize = cats.iter().map(|c| c.targets.len()).sum();
    let (mut last_done, mut last_total) = (0, 0);
    scan_with_progress(
        &env.ctx(),
        &cats,
        |_| None,
        |_, done, total, _| {
            last_done = done;
            last_total = total;
        },
    );
    assert_eq!(last_total, total_targets);
    assert_eq!(last_done, total_targets);
}
