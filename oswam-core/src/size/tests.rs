use super::*;
use crate::fsops::fake::FakeFs;

fn bytes(fs: &FakeFs, path: &str) -> u64 {
    physical_size(fs, Path::new(path)).unwrap()
}

fn one(fs: &FakeFs, path: &str) -> Measure {
    measure(fs, Path::new(path)).unwrap()
}

#[test]
fn sums_directory_tree_physically() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/a", "/c/sub"]);
    fs.file("/c/a", 4);
    fs.dir("/c/sub", &["/c/sub/b"]);
    fs.file("/c/sub/b", 16);
    assert_eq!(bytes(&fs, "/c"), (8 + 4 + 8 + 16) * 512);
}

#[test]
fn inaccessible_child_is_skipped_not_fatal() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/ok", "/c/denied"]);
    fs.file("/c/ok", 4);
    assert_eq!(bytes(&fs, "/c"), 4 * 512);
}

#[test]
fn an_unreadable_child_keeps_its_parent() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/ok", "/c/denied"]);
    fs.file("/c/ok", 4);
    assert!(one(&fs, "/c").kept);
}

#[test]
fn kept_travels_up_from_a_grandchild() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/sub"]);
    fs.dir("/c/sub", &["/c/sub/a", "/c/sub/keep"]);
    fs.file("/c/sub/a", 4);
    fs.file("/c/sub/keep", 6);
    let m = measure_where(&fs, Path::new("/c"), &|p, _| p != Path::new("/c/sub/keep")).unwrap();
    assert!(m.kept);
    assert_eq!(m.bytes, 4 * 512);
}

#[test]
fn contents_measure_leaves_out_the_directory_itself() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/a"]);
    fs.file("/c/a", 4);
    let m = measure_contents(&fs, Path::new("/c"), &|_, _| true).unwrap();
    assert_eq!(m.bytes, 4 * 512);
}

#[test]
fn inaccessible_child_marks_the_measure_partial() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/ok", "/c/denied"]);
    fs.file("/c/ok", 4);
    assert!(one(&fs, "/c").partial);
}

#[test]
fn fully_readable_tree_is_not_partial() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/a"]);
    fs.file("/c/a", 4);
    assert!(!one(&fs, "/c").partial);
}

#[test]
fn a_kept_child_leaves_its_parent_out_of_the_count() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/a", "/c/keep"]);
    fs.file("/c/a", 4);
    fs.file("/c/keep", 6);
    let m = measure_where(&fs, Path::new("/c"), &|p, _| p != Path::new("/c/keep")).unwrap();
    assert_eq!(m.bytes, 4 * 512);
}

#[test]
fn nothing_kept_counts_the_parent_too() {
    let mut fs = FakeFs::default();
    fs.dir("/c", &["/c/a"]);
    fs.file("/c/a", 4);
    let m = measure_where(&fs, Path::new("/c"), &|_, _| true).unwrap();
    assert_eq!(m.bytes, (8 + 4) * 512);
}

#[test]
fn single_file() {
    let mut fs = FakeFs::default();
    fs.file("/f", 3);
    assert_eq!(bytes(&fs, "/f"), 1536);
}

fn linked_tree() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir("/a", &["/a/x"]);
    fs.dir("/b", &["/b/x"]);
    fs.hardlink("/a/x", 100, 7);
    fs.hardlink("/b/x", 100, 7);
    fs
}

#[test]
fn a_hardlink_promises_only_its_share_of_the_blocks() {
    let fs = linked_tree();
    assert_eq!(bytes(&fs, "/a"), (8 + 50) * 512);
    assert_eq!(bytes(&fs, "/b"), (8 + 50) * 512);
}

#[test]
fn a_measure_never_depends_on_what_was_measured_before() {
    let fs = linked_tree();
    assert_eq!(bytes(&fs, "/a"), bytes(&fs, "/b"));
}

#[test]
fn two_links_to_one_inode_together_count_it_once() {
    let mut fs = FakeFs::default();
    fs.dir("/d", &["/d/one", "/d/two"]);
    fs.hardlink("/d/one", 50, 9);
    fs.hardlink("/d/two", 50, 9);
    assert_eq!(bytes(&fs, "/d"), (8 + 50) * 512);
}
