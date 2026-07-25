use super::*;
use crate::category::CleanupKind;
use crate::risk::RiskLevel;

const AUDITED_MEASURED: [&str; 1] = ["~/.npm/_cacache"];

const REFETCHED: [&str; 9] = [
    "~/.cache",
    "~/.npm/_cacache",
    "~/.yarn/berry/cache",
    "~/.gradle/caches",
    "~/.gradle/wrapper",
    "~/.cargo/registry/cache",
    "~/.bun/install/cache",
    "~/.pub-cache",
    "~/.nuget/packages",
];

#[test]
fn no_builtin_target_is_marked_never() {
    assert!(all_targets().iter().all(|t| t.risk < RiskLevel::Never));
}

#[test]
fn a_cache_that_costs_a_download_is_never_preselected_as_safe() {
    for path in REFETCHED {
        let target = all_targets()
            .into_iter()
            .find(|t| t.path == path)
            .unwrap_or_else(|| panic!("{path} is gone from the registry"));
        assert!(
            target.risk >= RiskLevel::Caution,
            "{path} is auto-selected as Safe but costs a re-download"
        );
    }
}

#[test]
fn a_measured_native_is_both_probed_and_pathful() {
    for target in all_targets().iter().filter(|t| t.measured) {
        let spec = target
            .native
            .as_ref()
            .unwrap_or_else(|| panic!("{}: measured is meaningless without a native", target.path));
        assert!(
            !spec.probe.is_empty(),
            "{}: a measured native must vanish when its tool is absent",
            target.path
        );
        assert!(
            target.path.starts_with('~') || target.path.starts_with('/'),
            "{}: a measured native needs a real path to size",
            target.path
        );
        assert_eq!(target.kind, CleanupKind::NativeCommand, "{}", target.path);
    }
}

#[test]
fn every_measured_native_is_reachable_from_the_scan() {
    assert!(all_targets().iter().any(|t| t.measured));
}

#[test]
fn a_measured_native_is_audited_to_clean_the_very_path_it_measured() {
    let measured: Vec<String> = all_targets()
        .iter()
        .filter(|t| t.measured)
        .map(|t| t.path.clone())
        .collect();
    for path in &measured {
        assert!(
            AUDITED_MEASURED.contains(&path.as_str()),
            "{path}: a measured native promises the size of this path — prove its clean command \
             addresses that path (not a project-local or cwd-dependent one), then list it here"
        );
    }
    assert_eq!(measured.len(), AUDITED_MEASURED.len());
}

#[test]
fn a_download_cache_name_is_recognised_wherever_it_is_enumerated() {
    for name in ["homebrew", "go-build", "pip", "cocoapods"] {
        assert!(
            crate::refetch::REFETCHED_CACHE_NAMES.contains(&name),
            "{name} must keep its Caution floor when enumerated under a Safe parent"
        );
    }
    for target in all_targets().iter().filter(|t| t.enumerate) {
        assert!(
            !crate::refetch::costs_a_download(std::path::Path::new(&target.path)),
            "{}: enumerate only its children, never the download cache itself",
            target.path
        );
    }
}
