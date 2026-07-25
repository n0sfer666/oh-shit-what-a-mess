use super::super::*;
use super::{one, Env};
use crate::category::builtin_categories;
use crate::fsops::fake::FakeFs;

#[test]
fn a_measured_native_survives_a_relocated_cache_without_promising_a_size() {
    let env = Env::new(FakeFs::default());
    let cats = one(
        "dev",
        crate::registry::native_at(
            "~/.npm/_cacache",
            &[&["npm", "--version"]],
            &["npm", "cache", "clean", "--force"],
            RiskLevel::Caution,
        ),
    );
    let res = scan(&env.ctx(), &cats, |_| Some(0));
    let entry = &res.categories[0].entries[0];
    assert!(entry.size_unknown);
    assert_eq!(entry.kind, CleanupKind::NativeCommand);
    assert_eq!(res.total_bytes, 0);
}

#[test]
fn native_uses_estimator_and_serializes() {
    let env = Env::new(FakeFs::default());
    let res = scan(&env.ctx(), &builtin_categories(), |spec| {
        (spec.clean.first().map(String::as_str) == Some("docker")).then_some(25 * 1024)
    });
    let containers = res
        .categories
        .iter()
        .find(|c| c.id == "containers")
        .expect("containers");
    let docker = containers
        .entries
        .iter()
        .find(|e| e.kind == CleanupKind::NativeCommand)
        .expect("docker entry");
    assert_eq!(docker.physical_bytes, 25 * 1024);
    let json = serde_json::to_string(&res).expect("json");
    assert!(json.contains("\"risk\":\"caution\""));
}
