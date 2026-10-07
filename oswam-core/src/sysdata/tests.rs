use super::*;
use crate::category::CleanupKind;
use crate::config::Config;
use crate::fsops::fake::FakeFs;
use crate::fsops::{Meta, SF_RESTRICTED};
use crate::risk::RiskLevel;
use std::path::{Path, PathBuf};

const HOME: &str = "/Users/tester";

fn result_with(categories: Vec<ScanCategory>) -> ScanResult {
    let total_bytes = categories.iter().map(|c| c.total_bytes).sum();
    ScanResult {
        categories,
        total_bytes,
    }
}

fn category(id: &str, total_bytes: u64) -> ScanCategory {
    ScanCategory {
        id: id.into(),
        name: id.into(),
        glyph: String::new(),
        entries: Vec::new(),
        total_bytes,
    }
}

fn caches() -> Vec<ScanEntry> {
    let mut fs = FakeFs::default();
    fs.dir(ROOT_CACHES, &["/Library/Caches/app", "/Library/Caches/sip"]);
    fs.dir("/Library/Caches/app", &["/Library/Caches/app/blob"]);
    fs.file("/Library/Caches/app/blob", 10);
    fs.dir("/Library/Caches/sip", &[]);
    fs.entries.insert(
        PathBuf::from("/Library/Caches/sip"),
        Meta {
            is_dir: true,
            flags: SF_RESTRICTED,
            ..Meta::default()
        },
    );
    root_caches(&fs, &Config::default(), Path::new(HOME))
}

#[test]
fn root_caches_are_named_by_full_path_so_they_never_pass_for_user_caches() {
    let entries = caches();
    let app = entries
        .iter()
        .find(|e| e.path.ends_with("app"))
        .expect("app");
    assert_eq!(app.display, "/Library/Caches/app");
    assert_eq!(app.risk, RiskLevel::Danger);
    assert!(app.needs_root && app.permanent_only);
}

#[test]
fn a_protected_root_cache_is_shown_but_not_offered() {
    let entries = caches();
    let sip = entries
        .iter()
        .find(|e| e.path.ends_with("sip"))
        .expect("sip");
    assert_eq!(sip.kind, CleanupKind::InfoOnly);
    assert_eq!(sip.risk, RiskLevel::Never);
}

#[test]
fn absorbed_entries_count_only_what_can_be_deleted() {
    let mut result = result_with(vec![category(SYSTEM_DATA_ID, 100), category("dev", 50)]);
    let entries = caches();
    let deletable = entries
        .iter()
        .find(|e| e.path.ends_with("app"))
        .expect("app")
        .physical_bytes;
    absorb(&mut result, entries);
    let system = &result.categories[0];
    assert_eq!(system.entries.len(), 2);
    assert_eq!(system.total_bytes, 100 + deletable);
    assert_eq!(result.total_bytes, 150 + deletable);
}

#[test]
fn absorbing_without_a_system_data_category_creates_it_instead_of_dropping_entries() {
    let mut result = result_with(vec![category("dev", 50)]);
    absorb(&mut result, caches());
    assert_eq!(result.categories[0].id, SYSTEM_DATA_ID);
    assert_eq!(result.categories[0].name, "System Data (macOS)");
    assert_eq!(result.categories[0].entries.len(), 2);
}

#[test]
fn absorbing_nothing_changes_nothing() {
    let mut result = result_with(vec![category("dev", 50)]);
    absorb(&mut result, Vec::new());
    assert_eq!(result.categories.len(), 1);
    assert_eq!(result.total_bytes, 50);
}
