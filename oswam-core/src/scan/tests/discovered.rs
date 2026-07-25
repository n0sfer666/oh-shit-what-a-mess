use super::*;
use crate::category::CleanupKind;
use crate::risk::RiskLevel;

fn artifacts(root: &str) -> Target {
    Target::discovered(
        root,
        vec!["node_modules"],
        4,
        CleanupKind::DeletePath,
        RiskLevel::Caution,
    )
}

fn scan_roots(fs: FakeFs, roots: &[&str]) -> ScanResult {
    scan_roots_with(Env::new(fs), roots)
}

fn scan_roots_with(env: Env, roots: &[&str]) -> ScanResult {
    let cats = vec![Category {
        id: "projects",
        name: "N",
        glyph: "g",
        targets: roots.iter().map(|r| artifacts(r)).collect(),
    }];
    env.scan(&cats)
}

fn project(fs: &mut FakeFs, dir: &str) {
    fs.dir(dir, &[&format!("{dir}/p")]);
    fs.dir(&format!("{dir}/p"), &[&format!("{dir}/p/node_modules")]);
    fs.dir(&format!("{dir}/p/node_modules"), &[]);
}

#[test]
fn two_roots_pointing_at_one_place_are_counted_once() {
    let mut fs = FakeFs::default();
    project(&mut fs, "/Users/tester/Developer");
    fs.link("/Users/tester/dev", "/Users/tester/Developer");
    let res = scan_roots(fs, &["~/Developer", "~/dev"]);
    let entries = &res.categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].path,
        PathBuf::from("/Users/tester/Developer/p/node_modules")
    );
    assert_eq!(res.total_bytes, entries[0].physical_bytes);
}

#[test]
fn a_root_nested_inside_another_root_is_not_counted_twice() {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/Developer",
        &["/Users/tester/Developer/Projects"],
    );
    project(&mut fs, "/Users/tester/Developer/Projects");
    fs.link("/Users/tester/Projects", "/Users/tester/Developer/Projects");
    let res = scan_roots(fs, &["~/Developer", "~/Projects"]);
    assert_eq!(res.categories[0].entries.len(), 1);
}

#[test]
fn a_root_symlinked_onto_another_volume_is_still_scanned() {
    let mut fs = FakeFs::default();
    project(&mut fs, "/Volumes/Work/dev");
    fs.rooted_at("/Users/tester");
    fs.link("/Users/tester/dev", "/Volumes/Work/dev");
    let entries = &scan_roots(fs, &["~/dev"]).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].path,
        PathBuf::from("/Volumes/Work/dev/p/node_modules")
    );
    assert_eq!(entries[0].display, "dev/p/node_modules");
    assert!(entries[0].permanent_only);
}

#[test]
fn protection_written_in_alias_form_survives_canonicalization() {
    let mut fs = FakeFs::default();
    project(&mut fs, "/Users/tester/work");
    fs.link("/Users/tester/code", "/Users/tester/work");
    let env = Env::protecting(fs, "~/code/p");
    let entries = &scan_roots_with(env, &["~/code"]).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].path,
        PathBuf::from("/Users/tester/work/p/node_modules")
    );
    assert_eq!(entries[0].risk, RiskLevel::Never);
}

#[test]
fn protection_of_the_real_path_survives_an_aliased_root() {
    let mut fs = FakeFs::default();
    project(&mut fs, "/Users/tester/work");
    fs.link("/Users/tester/code", "/Users/tester/work");
    let env = Env::protecting(fs, "~/work/p");
    let entries = &scan_roots_with(env, &["~/code"]).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].risk, RiskLevel::Never);
}
