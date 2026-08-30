mod proof;

use super::*;
use crate::fsops::fake::FakeFs;

pub(super) fn projects_tree() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/a", "/dev/b"]);
    fs.dir("/dev/a", &["/dev/a/node_modules", "/dev/a/src"]);
    fs.dir("/dev/a/node_modules", &["/dev/a/node_modules/nested"]);
    fs.dir(
        "/dev/a/node_modules/nested",
        &["/dev/a/node_modules/nested/node_modules"],
    );
    fs.dir("/dev/a/node_modules/nested/node_modules", &[]);
    fs.dir("/dev/a/src", &[]);
    fs.dir("/dev/b", &["/dev/b/target", "/dev/b/Cargo.toml"]);
    fs.file("/dev/b/Cargo.toml", 1);
    fs.dir("/dev/b/target", &[]);
    fs
}

#[test]
fn finds_named_dirs_across_projects() {
    let fs = projects_tree();
    let found = find_named_dirs(&fs, Path::new("/dev"), &["node_modules", "target"], 5);
    assert!(found.contains(&PathBuf::from("/dev/a/node_modules")));
    assert!(found.contains(&PathBuf::from("/dev/b/target")));
}

#[test]
fn prunes_nested_matches() {
    let fs = projects_tree();
    let found = find_named_dirs(&fs, Path::new("/dev"), &["node_modules"], 5);
    assert_eq!(found.len(), 1);
    assert!(!found.contains(&PathBuf::from("/dev/a/node_modules/nested/node_modules")));
}

#[test]
fn respects_max_depth() {
    let fs = projects_tree();
    let found = find_named_dirs(&fs, Path::new("/dev"), &["node_modules"], 0);
    assert!(found.is_empty());
}

#[test]
fn bare_star_matches_nothing() {
    let fs = projects_tree();
    assert!(find_named_dirs(&fs, Path::new("/dev"), &["*"], 5).is_empty());
}

#[test]
fn skips_symlinked_dirs() {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/link"]);
    fs.entries.insert(
        PathBuf::from("/dev/link"),
        crate::fsops::Meta {
            is_dir: true,
            is_symlink: true,
            ..crate::fsops::Meta::default()
        },
    );
    fs.children.insert(
        PathBuf::from("/dev/link"),
        vec![PathBuf::from("/dev/link/node_modules")],
    );
    fs.dir("/dev/link/node_modules", &[]);
    let found = find_named_dirs(&fs, Path::new("/dev"), &["node_modules"], 5);
    assert!(found.is_empty());
}

#[test]
fn a_symlinked_root_is_still_walked() {
    let mut fs = FakeFs::default();
    fs.linked_dir(
        "/Users/tester/dev",
        "/Volumes/Work/dev",
        &["/Users/tester/dev/p"],
    );
    fs.dir("/Users/tester/dev/p", &["/Users/tester/dev/p/node_modules"]);
    fs.dir("/Users/tester/dev/p/node_modules", &[]);
    assert_eq!(
        find_named_dirs(&fs, Path::new("/Users/tester/dev"), &["node_modules"], 5),
        vec![PathBuf::from("/Users/tester/dev/p/node_modules")]
    );
}
