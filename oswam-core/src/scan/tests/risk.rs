use super::super::*;
use super::{one, Env};
use crate::category::Target;
use crate::fsops::fake::FakeFs;

fn caches_with(children: &[&str]) -> FakeFs {
    let mut fs = FakeFs::default();
    let paths: Vec<String> = children
        .iter()
        .map(|c| format!("/Users/tester/Library/Caches/{c}"))
        .collect();
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    fs.dir("/Users/tester/Library/Caches", &refs);
    for path in &paths {
        fs.dir(path, &[]);
    }
    fs
}

fn caches_cats() -> Vec<Category> {
    one(
        "system",
        Target::enumerated(
            "~/Library/Caches",
            CleanupKind::DeleteContents,
            RiskLevel::Safe,
        ),
    )
}

#[test]
fn a_download_cache_child_is_never_offered_as_safe() {
    let res = Env::new(caches_with(&["Homebrew", "com.apple.Safari"])).scan(&caches_cats());
    let risk = |name: &str| {
        res.categories[0]
            .entries
            .iter()
            .find(|e| e.display == name)
            .unwrap_or_else(|| panic!("{name}"))
            .risk
    };
    assert_eq!(risk("Homebrew"), RiskLevel::Caution);
    assert_eq!(risk("com.apple.Safari"), RiskLevel::Safe);
}

#[test]
fn a_download_cache_in_use_is_not_pushed_all_the_way_to_danger() {
    let env = Env::holding(
        caches_with(&["go-build"]),
        "/Users/tester/Library/Caches/go-build/trim.txt",
    );
    assert_eq!(
        env.scan(&caches_cats()).categories[0].entries[0].risk,
        RiskLevel::Caution
    );
}

#[test]
fn an_info_only_entry_still_reports_that_the_user_protected_it() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/Library/Application Support/Claude", &[]);
    let cats = one(
        "big-data",
        Target::new(
            "~/Library/Application Support/Claude",
            CleanupKind::InfoOnly,
            RiskLevel::Danger,
        ),
    );
    let env = Env::protecting(fs, "~/Library/Application Support/Claude");
    assert_eq!(
        env.scan(&cats).categories[0].entries[0].risk,
        RiskLevel::Never
    );
}

fn root_owned_fs() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir("/Library/Logs", &[]);
    fs.entries.insert(
        PathBuf::from("/Library/Logs"),
        crate::fsops::Meta {
            is_dir: true,
            blocks: 8,
            uid: 0,
            ..crate::fsops::Meta::default()
        },
    );
    fs
}

fn root_cats() -> Vec<Category> {
    one(
        "elevated",
        Target::new(
            "/Library/Logs",
            CleanupKind::DeleteContents,
            RiskLevel::Caution,
        )
        .needing_root(),
    )
}

#[test]
fn root_targets_are_hidden_without_elevation() {
    let res = Env::new(root_owned_fs()).scan(&root_cats());
    assert!(res.categories[0].entries.is_empty());
}

#[test]
fn elevation_keeps_root_owned_targets_deletable() {
    let res = Env::elevated(root_owned_fs()).scan(&root_cats());
    assert_eq!(res.categories[0].entries[0].risk, RiskLevel::Caution);
}

#[test]
fn a_shared_blob_is_split_between_the_targets_that_hold_it() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/cache", &["/Users/tester/cache/blob"]);
    fs.hardlink("/Users/tester/cache/blob", 40, 77);
    fs.dir("/Users/tester/data", &["/Users/tester/data/blob"]);
    fs.hardlink("/Users/tester/data/blob", 40, 77);
    let cats = vec![Category {
        id: "c",
        name: "N",
        glyph: "g",
        targets: vec![
            Target::new("~/cache", CleanupKind::DeleteContents, RiskLevel::Safe),
            Target::new("~/data", CleanupKind::InfoOnly, RiskLevel::Danger),
        ],
    }];
    let res = Env::new(fs).scan(&cats);
    let cache = &res.categories[0].entries[0];
    let info = &res.categories[0].entries[1];
    assert_eq!(info.kind, CleanupKind::InfoOnly);
    assert_eq!(cache.physical_bytes, 20 * 512);
    assert_eq!(info.physical_bytes, (20 + 8) * 512);
}
