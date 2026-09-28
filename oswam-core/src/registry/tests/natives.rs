use super::*;
use crate::risk::RiskLevel;

#[test]
fn docker_prunes_are_split_per_resource() {
    for (head, expected) in [
        (
            &["docker", "image"][..],
            vec!["docker", "image", "prune", "-af"],
        ),
        (
            &["docker", "container"][..],
            vec!["docker", "container", "prune", "-f"],
        ),
        (
            &["docker", "builder"][..],
            vec!["docker", "builder", "prune", "-af"],
        ),
        (
            &["docker", "volume"][..],
            vec!["docker", "volume", "prune", "-af"],
        ),
    ] {
        assert_eq!(clean_of("containers", head), expected);
    }
}

#[test]
fn docker_estimate_filters_are_disjoint_and_complete() {
    let filters: Vec<Vec<String>> = cat("containers")
        .targets
        .iter()
        .filter_map(|t| t.native.as_ref())
        .map(|s| s.estimate_filter.clone())
        .collect();
    assert_eq!(filters.len(), 4);
    let mut flat: Vec<String> = filters.into_iter().flatten().collect();
    let total = flat.len();
    flat.sort();
    flat.dedup();
    assert_eq!(flat.len(), total);
    assert_eq!(
        flat,
        vec!["Build Cache", "Containers", "Images", "Local Volumes"]
    );
}

#[test]
fn volume_prune_is_the_only_dangerous_docker_step() {
    for t in cat("containers")
        .targets
        .iter()
        .filter(|t| t.native.is_some())
    {
        let volumes = t.path.contains("volume");
        assert_eq!(t.risk >= RiskLevel::Danger, volumes, "{}", t.path);
    }
}

#[test]
fn simulator_runtimes_are_pruned_wholesale_or_by_age() {
    assert_eq!(
        clean_of(
            "simulators",
            &["xcrun", "simctl", "runtime", "delete", "all"]
        ),
        vec!["xcrun", "simctl", "runtime", "delete", "all"]
    );
    assert_eq!(
        clean_of(
            "simulators",
            &["xcrun", "simctl", "runtime", "delete", "--notUsedSinceDays"]
        ),
        vec![
            "xcrun",
            "simctl",
            "runtime",
            "delete",
            "--notUsedSinceDays",
            "180"
        ]
    );
}

#[test]
fn simctl_runtime_delete_never_claims_the_device_only_unavailable_alias() {
    for spec in cat("simulators")
        .targets
        .iter()
        .filter_map(|t| t.native.as_ref())
        .filter(|s| s.clean.get(2).is_some_and(|w| w == "runtime"))
    {
        assert!(
            !spec.clean.iter().any(|w| w == "unavailable"),
            "{:?}: simctl runtime delete rejects 'unavailable' — it exists only for devices",
            spec.clean
        );
    }
}

#[test]
fn only_the_wholesale_runtime_delete_promises_a_size() {
    let sized: Vec<Vec<String>> = cat("simulators")
        .targets
        .iter()
        .filter_map(|t| t.native.as_ref())
        .filter(|s| !s.estimate.is_empty())
        .map(|s| s.clean.clone())
        .collect();
    assert_eq!(
        sized,
        vec![vec![
            "xcrun".to_string(),
            "simctl".to_string(),
            "runtime".to_string(),
            "delete".to_string(),
            "all".to_string(),
        ]],
        "a size shown next to a partial prune would promise more than it deletes"
    );
}

#[test]
fn the_runtime_size_is_read_from_the_disk_image_total() {
    let spec = cat("simulators")
        .targets
        .iter()
        .filter_map(|t| t.native.as_ref())
        .find(|s| !s.estimate.is_empty())
        .expect("a sized simulator native")
        .clone();
    assert_eq!(spec.estimate, vec!["xcrun", "simctl", "runtime", "list"]);
    assert_eq!(spec.estimate_filter, vec!["Total Disk Images"]);
}

#[test]
fn go_module_cache_is_cleaned_by_go_itself() {
    assert_eq!(
        clean_of("dev", &["go", "clean", "-modcache"]),
        vec!["go", "clean", "-modcache"]
    );
    assert!(
        !all_targets()
            .iter()
            .any(|t| t.measured && t.path.contains("go/pkg/mod")),
        "GOMODCACHE moves with the environment: never size a hardcoded default and let go clean \
         wipe somewhere else"
    );
}

#[test]
fn uv_tools_are_uninstalled_by_uv_so_the_shims_go_too() {
    assert_eq!(
        clean_of("dev", &["uv", "tool", "uninstall"]),
        vec!["uv", "tool", "uninstall", "--all"]
    );
    assert!(target("dev", "~/.local/share/uv/tools").measured);
}
