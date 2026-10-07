mod catalog;
mod claude;
mod honesty;
mod invariants;
mod natives;
mod nesting;
mod system_data;

use super::*;

fn cat(id: &str) -> Category {
    builtin_categories()
        .into_iter()
        .find(|c| c.id == id)
        .expect("category")
}

fn all_targets() -> Vec<Target> {
    builtin_categories()
        .into_iter()
        .flat_map(|c| c.targets)
        .collect()
}

fn deletes(target: &Target) -> bool {
    matches!(
        target.kind,
        CleanupKind::DeleteContents | CleanupKind::DeletePath
    )
}

fn natives() -> Vec<NativeSpec> {
    all_targets().into_iter().filter_map(|t| t.native).collect()
}

fn target(id: &str, path: &str) -> Target {
    cat(id)
        .targets
        .into_iter()
        .find(|t| t.path == path)
        .unwrap_or_else(|| panic!("{id}: {path}"))
}

fn has_path(id: &str, path: &str) -> bool {
    cat(id).targets.iter().any(|t| t.path == path)
}

fn clean_of(id: &str, head: &[&str]) -> Vec<String> {
    cat(id)
        .targets
        .iter()
        .filter_map(|t| t.native.as_ref())
        .find(|s| s.clean.iter().zip(head).all(|(a, b)| a == b))
        .unwrap_or_else(|| panic!("{id}: {head:?}"))
        .clean
        .clone()
}

#[test]
fn has_expected_categories() {
    let ids: Vec<&str> = builtin_categories().iter().map(|c| c.id).collect();
    assert_eq!(
        ids,
        vec![
            "system-data",
            "dev",
            "simulators",
            "containers",
            "browsers",
            "claude",
            "projects",
            "big-data",
            "trash",
        ]
    );
}
