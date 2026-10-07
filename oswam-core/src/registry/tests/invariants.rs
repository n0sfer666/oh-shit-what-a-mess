use super::*;
use crate::risk::RiskLevel;

const VM_IMAGE_SUFFIXES: [&str; 6] = [".raw", ".qcow2", ".vdi", ".vmdk", ".img", ".sparsebundle"];

const REDOWNLOADED_VM_IMAGES: [&str; 1] =
    ["~/Library/Application Support/Claude/vm_bundles/claudevm.bundle/rootfs.img"];

const VM_BUNDLE_DIRS: [&str; 3] = ["vm_bundles", ".bundle", "virtual machines.localized"];

fn words(spec: &crate::category::NativeSpec) -> Vec<&String> {
    spec.clean
        .iter()
        .chain(spec.estimate.iter())
        .chain(spec.probe.iter().flatten())
        .collect()
}

#[test]
fn no_target_is_bound_to_a_concrete_user() {
    for target in all_targets() {
        assert!(
            !target.path.contains("/Users/") && !target.path.contains("/home/"),
            "{} hardcodes a home directory instead of ~",
            target.path
        );
    }
    for spec in natives() {
        for arg in words(&spec) {
            assert!(
                !arg.contains("/Users/") && !arg.contains("/home/"),
                "native argument {arg} hardcodes a home directory"
            );
        }
    }
}

#[test]
fn no_target_points_at_a_vm_image() {
    for t in all_targets() {
        if REDOWNLOADED_VM_IMAGES.contains(&t.path.as_str()) {
            assert!(
                t.blocking_app.is_some() && t.risk >= RiskLevel::Caution,
                "{}: a re-downloadable VM image goes only while its app is closed",
                t.path
            );
            continue;
        }
        let lower = t.path.to_ascii_lowercase();
        assert!(
            !VM_IMAGE_SUFFIXES.iter().any(|s| lower.ends_with(s)) || !deletes(&t),
            "{}",
            t.path
        );
    }
}

#[test]
fn no_target_deletes_the_folder_a_vm_image_lives_in() {
    for t in all_targets() {
        let lower = t.path.to_ascii_lowercase();
        assert!(
            !VM_BUNDLE_DIRS.iter().any(|s| lower.ends_with(s)) || !deletes(&t),
            "{}: deleting this folder would rm a live VM disk — show it, do not clean it",
            t.path
        );
    }
}

#[test]
fn natives_never_shell_out_to_raw_state() {
    for spec in natives() {
        for arg in words(&spec) {
            let lower = arg.to_ascii_lowercase();
            assert!(
                !VM_IMAGE_SUFFIXES.iter().any(|s| lower.contains(s)),
                "{arg}"
            );
            assert!(!lower.contains("sh -c") && arg != "sh", "{arg}");
        }
    }
}

#[test]
fn natives_always_say_how_to_size_or_detect_themselves() {
    for spec in natives() {
        assert!(!spec.clean.is_empty(), "a native without a clean command");
        assert!(
            !spec.estimate.is_empty() || !spec.probe.is_empty(),
            "{:?} declares neither an estimate nor a probe",
            spec.clean
        );
    }
}

#[test]
fn a_native_is_detected_through_the_very_tool_the_cleanup_uses() {
    for spec in natives() {
        let tool = spec.clean.first().expect("clean program");
        let detector = spec
            .probe
            .last()
            .and_then(|last| last.first())
            .or_else(|| spec.estimate.first())
            .expect("probe or estimate program");
        assert_eq!(
            detector, tool,
            "detection must fail when {tool} is absent or unusable"
        );
    }
}

#[test]
fn a_probe_never_installs_anything_on_its_way_to_an_answer() {
    for spec in natives().iter().filter(|s| !s.probe.is_empty()) {
        let asks_xcrun = spec
            .probe
            .iter()
            .any(|c| c.first().is_some_and(|p| p == "xcrun"));
        if !asks_xcrun {
            continue;
        }
        let first = spec.probe.first().and_then(|c| c.first()).expect("program");
        assert_eq!(
            first, "xcode-select",
            "xcrun opens the Command Line Tools installer unless a developer dir is known"
        );
    }
}

#[test]
fn grouping_is_opt_in_and_only_for_enumerated_targets() {
    assert!(all_targets()
        .iter()
        .all(|t| !t.group_by_stem || t.enumerate));
    assert_eq!(all_targets().iter().filter(|t| t.group_by_stem).count(), 1);
}

#[test]
fn project_roots_are_searched_below_the_top_level() {
    for t in &cat("projects").targets {
        assert!(t.discover.is_some() && t.depth > 0, "{}", t.path);
    }
}

#[test]
fn discover_targets_name_no_bare_wildcard() {
    for t in all_targets().iter().filter(|t| t.discover.is_some()) {
        let names = t.discover.as_ref().expect("discover");
        assert!(
            names.iter().all(|n| *n != "*" && !n.is_empty()),
            "{}",
            t.path
        );
    }
}

#[test]
fn a_native_runs_as_root_only_when_the_target_demands_it() {
    for t in all_targets().iter().filter(|t| t.native.is_some()) {
        let spec = t.native.as_ref().expect("native");
        assert_eq!(
            spec.privileged, t.needs_root,
            "{}: a user-level command must drop back to the caller under sudo",
            t.path
        );
    }
}

#[test]
fn only_system_data_reaches_for_root() {
    for c in builtin_categories()
        .iter()
        .filter(|c| c.id != SYSTEM_DATA_ID)
    {
        assert!(c.targets.iter().all(|t| !t.needs_root), "{}", c.id);
    }
}

#[test]
fn an_app_bound_target_is_never_a_native_command() {
    for t in all_targets().iter().filter(|t| t.blocking_app.is_some()) {
        assert!(t.native.is_none(), "{}", t.path);
    }
}
