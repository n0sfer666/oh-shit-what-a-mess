use super::super::*;
use super::projects_tree;
use crate::fsops::fake::FakeFs;

fn build_tree() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/p"]);
    fs.dir(
        "/dev/p",
        &[
            "/dev/p/build-s7",
            "/dev/p/build-scripts",
            "/dev/p/build",
            "/dev/p/buildsrc",
        ],
    );
    fs.dir("/dev/p/build-s7", &["/dev/p/build-s7/CMakeCache.txt"]);
    fs.file("/dev/p/build-s7/CMakeCache.txt", 1);
    fs.dir("/dev/p/build-scripts", &["/dev/p/build-scripts/deploy.sh"]);
    fs.file("/dev/p/build-scripts/deploy.sh", 1);
    fs.dir("/dev/p/build", &[]);
    fs.dir("/dev/p/buildsrc", &[]);
    fs
}

#[test]
fn prefix_pattern_takes_generated_dirs() {
    let fs = build_tree();
    let found = find_named_dirs(&fs, Path::new("/dev"), &["build", "build-*"], 5);
    assert!(found.contains(&PathBuf::from("/dev/p/build-s7")));
    assert!(!found.contains(&PathBuf::from("/dev/p/buildsrc")));
}

#[test]
fn prefix_pattern_spares_source_dirs() {
    let fs = build_tree();
    let found = find_named_dirs(&fs, Path::new("/dev"), &["build-*"], 5);
    assert!(!found.contains(&PathBuf::from("/dev/p/build-scripts")));
}

#[test]
fn handwritten_makefile_does_not_mark_a_dir_as_generated() {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/p"]);
    fs.dir("/dev/p", &["/dev/p/build-scripts"]);
    fs.dir("/dev/p/build-scripts", &["/dev/p/build-scripts/Makefile"]);
    fs.file("/dev/p/build-scripts/Makefile", 1);
    let found = find_named_dirs(&fs, Path::new("/dev"), &["build-*"], 5);
    assert!(found.is_empty());
}

#[test]
fn exact_name_needs_no_marker() {
    let fs = projects_tree();
    let found = find_named_dirs(&fs, Path::new("/dev"), &["node_modules"], 5);
    assert_eq!(found, vec![PathBuf::from("/dev/a/node_modules")]);
}

#[test]
fn a_build_folder_without_a_generation_marker_is_left_alone() {
    let fs = build_tree();
    assert!(find_named_dirs(&fs, Path::new("/dev"), &["build"], 5).is_empty());
}

#[test]
fn a_build_folder_that_marks_itself_generated_is_an_artifact() {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/p"]);
    fs.dir("/dev/p", &["/dev/p/build"]);
    fs.dir("/dev/p/build", &["/dev/p/build/CMakeCache.txt"]);
    fs.file("/dev/p/build/CMakeCache.txt", 1);
    assert_eq!(
        find_named_dirs(&fs, Path::new("/dev"), &["build"], 5),
        vec![PathBuf::from("/dev/p/build")]
    );
}

#[test]
fn a_manifest_next_door_does_not_vouch_for_a_build_folder() {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/p"]);
    fs.dir("/dev/p", &["/dev/p/build", "/dev/p/Cargo.toml"]);
    fs.file("/dev/p/Cargo.toml", 1);
    fs.dir("/dev/p/build", &["/dev/p/build/main.cpp"]);
    fs.file("/dev/p/build/main.cpp", 1);
    assert!(find_named_dirs(&fs, Path::new("/dev"), &["build"], 5).is_empty());
}

#[test]
fn a_target_folder_without_a_build_system_is_left_alone() {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/ml"]);
    fs.dir("/dev/ml", &["/dev/ml/target", "/dev/ml/train.py"]);
    fs.file("/dev/ml/train.py", 1);
    fs.dir("/dev/ml/target", &["/dev/ml/target/labels.csv"]);
    fs.file("/dev/ml/target/labels.csv", 4);
    assert!(find_named_dirs(&fs, Path::new("/dev"), &["target"], 5).is_empty());
}

#[test]
fn a_target_folder_next_to_a_manifest_is_an_artifact() {
    let fs = projects_tree();
    assert_eq!(
        find_named_dirs(&fs, Path::new("/dev"), &["target"], 5),
        vec![PathBuf::from("/dev/b/target")]
    );
}

#[test]
fn a_target_folder_that_marks_itself_generated_needs_no_manifest() {
    let mut fs = FakeFs::default();
    fs.dir("/dev", &["/dev/p"]);
    fs.dir("/dev/p", &["/dev/p/target"]);
    fs.dir("/dev/p/target", &["/dev/p/target/CACHEDIR.TAG"]);
    fs.file("/dev/p/target/CACHEDIR.TAG", 1);
    assert_eq!(
        find_named_dirs(&fs, Path::new("/dev"), &["target"], 5),
        vec![PathBuf::from("/dev/p/target")]
    );
}
