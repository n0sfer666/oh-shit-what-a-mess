use super::*;

#[test]
fn parses_units() {
    assert_eq!(parse_human_size("0B"), Some(0));
    assert_eq!(parse_human_size("1.5GB"), Some(1_500_000_000));
    assert_eq!(parse_human_size("100MB"), Some(100_000_000));
    assert_eq!(parse_human_size("2KiB"), Some(2048));
}

#[test]
fn rejects_garbage() {
    assert_eq!(parse_human_size("abc"), None);
    assert_eq!(parse_human_size("5XB"), None);
}

#[test]
fn reads_actually_reclaimed_space() {
    let out = "deleted: sha256:abc\n\nTotal reclaimed space: 1.503GB\n";
    assert_eq!(parse_reclaimed(out), Some(1_503_000_000));
}

#[test]
fn a_command_without_a_reclaimed_line_reports_nothing() {
    assert_eq!(parse_reclaimed("Store is already pruned\n"), None);
}

#[test]
fn sums_reclaimable_column() {
    let out = "1.5GB (50%)\n200MB (10%)\n0B (0%)\n";
    assert_eq!(parse_reclaimable(out), 1_700_000_000);
}

#[test]
fn unfiltered_sum_reads_typed_rows_too() {
    assert_eq!(
        parse_reclaimable(typed_df()),
        4_114_000_000 + 253_200_000 + 7_292_000_000
    );
}

fn typed_df() -> &'static str {
    "Images\t4.114GB (93%)\nContainers\t0B\nLocal Volumes\t253.2MB (100%)\nBuild Cache\t7.292GB\n"
}

#[test]
fn filters_reclaimable_by_type() {
    let only_build = vec!["Build Cache".to_string()];
    assert_eq!(
        parse_reclaimable_filtered(typed_df(), &only_build),
        7_292_000_000
    );
}

#[test]
fn filter_types_do_not_overlap() {
    let images = vec!["Images".to_string(), "Containers".to_string()];
    let volumes = vec!["Local Volumes".to_string()];
    let build = vec!["Build Cache".to_string()];
    let sum: u64 = [&images, &volumes, &build]
        .iter()
        .map(|f| parse_reclaimable_filtered(typed_df(), f))
        .sum();
    assert_eq!(sum, 4_114_000_000 + 253_200_000 + 7_292_000_000);
}

#[test]
fn unknown_type_yields_zero() {
    let none = vec!["Nope".to_string()];
    assert_eq!(parse_reclaimable_filtered(typed_df(), &none), 0);
}

fn spec(estimate: &[&str]) -> NativeSpec {
    NativeSpec {
        estimate: estimate.iter().map(|s| s.to_string()).collect(),
        clean: vec!["true".to_string()],
        ..NativeSpec::default()
    }
}

#[test]
fn missing_tool_hides_the_target() {
    clear_estimates();
    assert_eq!(estimate(&spec(&["oswam-no-such-binary"])), None);
}

#[test]
fn a_failing_estimate_hides_the_target_instead_of_calling_it_empty() {
    clear_estimates();
    assert_eq!(estimate(&spec(&["false"])), None);
}

fn probed(probe: &[&[&str]]) -> NativeSpec {
    NativeSpec {
        probe: probe
            .iter()
            .map(|c| c.iter().map(|s| s.to_string()).collect())
            .collect(),
        clean: vec!["true".to_string()],
        ..NativeSpec::default()
    }
}

#[test]
fn a_probe_that_exits_with_an_error_hides_the_target() {
    clear_estimates();
    assert_eq!(estimate(&probed(&[&["false"]])), None);
}

#[test]
fn a_missing_probe_tool_hides_the_target() {
    clear_estimates();
    assert_eq!(estimate(&probed(&[&["oswam-no-such-binary"]])), None);
}

#[test]
fn a_probed_target_without_an_estimator_shows_up_sizeless() {
    clear_estimates();
    assert_eq!(estimate(&probed(&[&["true"]])), Some(0));
}

#[test]
fn every_step_of_the_probe_chain_must_pass() {
    clear_estimates();
    assert_eq!(estimate(&probed(&[&["true"], &["false"]])), None);
    assert_eq!(estimate(&probed(&[&["false"], &["true"]])), None);
}

#[test]
fn a_probe_output_is_never_read_as_a_size() {
    clear_estimates();
    let spec = probed(&[&["echo", "Images\t9GB"]]);
    assert_eq!(estimate(&spec), Some(0));
}

#[test]
fn a_probe_gates_the_estimate_it_travels_with() {
    clear_estimates();
    let mut spec = spec(&["echo", "1.5GB"]);
    spec.probe = vec![vec!["false".to_string()]];
    assert_eq!(estimate(&spec), None);
    spec.probe = vec![vec!["true".to_string()]];
    assert_eq!(estimate(&spec), Some(1_500_000_000));
}
