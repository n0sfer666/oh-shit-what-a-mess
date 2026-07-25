use super::*;
use crate::category::CleanupKind;
use crate::risk::RiskLevel;

#[test]
fn caches_are_enumerated() {
    let caches = target("system", "~/Library/Caches");
    assert!(caches.enumerate);
    assert_eq!(caches.kind, CleanupKind::DeleteContents);
}

#[test]
fn podman_machine_is_info_only() {
    let podman = target("containers", "~/.local/share/containers/podman");
    assert_eq!(podman.kind, CleanupKind::InfoOnly);
    assert!(podman.risk >= RiskLevel::Danger);
}

#[test]
fn avd_dir_and_its_ini_are_one_unit() {
    let avd = target("simulators", "~/.android/avd");
    assert!(avd.group_by_stem);
    assert!(avd.enumerate);
}

#[test]
fn browsers_touch_only_cache_subfolders() {
    const ALLOWED: [&str; 9] = [
        "Cache",
        "Code Cache",
        "GPUCache",
        "CacheStorage",
        "ShaderCache",
        "GrShaderCache",
        "DawnGraphiteCache",
        "DawnWebGPUCache",
        "OptGuideOnDeviceModel",
    ];
    let browsers = cat("browsers");
    assert!(!browsers.targets.is_empty());
    for t in &browsers.targets {
        let names = t.discover.as_ref().expect("discover");
        assert!(names.iter().all(|n| ALLOWED.contains(n)));
        assert_eq!(t.kind, CleanupKind::DeletePath);
    }
}

#[test]
fn project_artifacts_match_prefix_patterns() {
    let names = cat("projects").targets[0]
        .discover
        .clone()
        .expect("discover");
    assert!(names.contains(&"cmake-build-*"));
    assert!(names.contains(&"node_modules"));
    assert!(names.contains(&"target"));
}

#[test]
fn ambiguous_source_directory_names_are_not_matched_exactly() {
    let names = cat("projects").targets[0]
        .discover
        .clone()
        .expect("discover");
    for ambiguous in ["build", "dist", "out", "lib", "bin"] {
        assert!(
            !names.contains(&ambiguous),
            "{ambiguous} matches hand-written sources as often as artifacts"
        );
    }
}

#[test]
fn project_roots_are_several_and_case_insensitively_unique() {
    let roots: Vec<String> = cat("projects")
        .targets
        .iter()
        .map(|t| t.path.to_ascii_lowercase())
        .collect();
    assert!(roots.len() > 1);
    let mut uniq = roots.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), roots.len(), "{roots:?}");
    assert!(roots.iter().all(|r| r.starts_with("~/")));
}

#[test]
fn big_data_deletions_are_never_below_caution() {
    assert!(cat("big-data")
        .targets
        .iter()
        .all(|t| t.risk >= RiskLevel::Caution));
    assert!(has_path(
        "big-data",
        "~/Library/Application Support/Steam/steamapps/downloading"
    ));
}

#[test]
fn irreplaceable_user_data_is_never_softer_than_danger() {
    for target in cat("big-data")
        .targets
        .iter()
        .filter(|t| t.kind == crate::category::CleanupKind::InfoOnly)
    {
        assert_eq!(
            target.risk,
            RiskLevel::Danger,
            "{} — данные пользователя, не перекачиваемый кэш",
            target.path
        );
    }
}

#[test]
fn elevated_targets_all_require_root() {
    let elevated = cat("elevated");
    assert!(!elevated.targets.is_empty());
    assert!(elevated.targets.iter().all(|t| t.needs_root));
    assert!(has_path(
        "elevated",
        "/Library/Developer/CoreSimulator/Caches/dyld"
    ));
}
