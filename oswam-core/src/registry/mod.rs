mod bigdata;
mod browsers;
mod claude;
mod containers;
mod dev;
mod projects;
mod simulators;
mod system_data;
#[cfg(test)]
mod tests;
mod trash;

use crate::category::{Category, CleanupKind, NativeSpec, Target};
use crate::risk::RiskLevel;

pub use system_data::ID as SYSTEM_DATA_ID;

pub fn builtin_categories() -> Vec<Category> {
    vec![
        system_data::category(),
        dev::category(),
        simulators::category(),
        containers::category(),
        browsers::category(),
        claude::category(),
        projects::category(),
        bigdata::category(),
        trash::category(),
    ]
}

fn words(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

fn commands(parts: &[&[&str]]) -> Vec<Vec<String>> {
    parts.iter().map(|c| words(c)).collect()
}

pub(crate) fn native(label: &str, probe: &[&[&str]], clean: &[&str], risk: RiskLevel) -> Target {
    spec_target(
        label,
        NativeSpec {
            probe: commands(probe),
            clean: words(clean),
            ..NativeSpec::default()
        },
        risk,
    )
}

pub(crate) fn native_filtered(
    label: &str,
    probe: &[&[&str]],
    estimate: &[&str],
    clean: &[&str],
    filter: &[&str],
    risk: RiskLevel,
) -> Target {
    spec_target(
        label,
        NativeSpec {
            probe: commands(probe),
            estimate: words(estimate),
            clean: words(clean),
            estimate_filter: words(filter),
            ..NativeSpec::default()
        },
        risk,
    )
}

pub(crate) fn native_at(path: &str, probe: &[&[&str]], clean: &[&str], risk: RiskLevel) -> Target {
    Target {
        measured: true,
        ..native(path, probe, clean, risk)
    }
}

fn spec_target(label: &str, spec: NativeSpec, risk: RiskLevel) -> Target {
    Target {
        native: Some(spec),
        ..Target::new(label, CleanupKind::NativeCommand, risk)
    }
}
