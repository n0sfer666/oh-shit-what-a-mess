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
fn simulator_runtimes_are_pruned_by_age_and_availability() {
    assert_eq!(
        clean_of(
            "simulators",
            &["xcrun", "simctl", "runtime", "delete", "unavailable"]
        ),
        vec!["xcrun", "simctl", "runtime", "delete", "unavailable"]
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
