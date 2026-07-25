use super::super::*;
use super::{one, Env};
use crate::category::Target;
use crate::fsops::fake::FakeFs;

fn avd_fs() -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir(
        "/Users/tester/.android/avd",
        &[
            "/Users/tester/.android/avd/Pixel.avd",
            "/Users/tester/.android/avd/Pixel.ini",
        ],
    );
    fs.dir("/Users/tester/.android/avd/Pixel.avd", &[]);
    fs.file("/Users/tester/.android/avd/Pixel.ini", 2);
    fs
}

fn avd_cats() -> Vec<Category> {
    one(
        "simulators",
        Target::grouped(
            "~/.android/avd",
            CleanupKind::DeletePath,
            RiskLevel::Caution,
        ),
    )
}

#[test]
fn grouped_target_pairs_companion_and_sums_size() {
    let entries = &Env::new(avd_fs()).scan(&avd_cats()).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display, "Pixel.avd");
    assert_eq!(
        entries[0].companions,
        vec![PathBuf::from("/Users/tester/.android/avd/Pixel.ini")]
    );
    assert_eq!(entries[0].physical_bytes, (8 + 2) * 512);
}

#[test]
fn a_protected_companion_makes_the_whole_unit_info_only() {
    let env = Env::protecting(avd_fs(), "~/.android/avd/Pixel.ini");
    let entries = &env.scan(&avd_cats()).categories[0].entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, CleanupKind::InfoOnly);
    assert!(entries[0].companions.is_empty());
}
