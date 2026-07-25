mod entry;
mod target;
#[cfg(test)]
mod tests;

use crate::category::{Category, CleanupKind, NativeSpec, Target};
use crate::config::Config;
use crate::fsops::FsOps;
use crate::guard::Guard;
use crate::process::ProcessProbe;
use crate::risk::RiskLevel;
use serde::Serialize;
use std::path::{Path, PathBuf};
use target::process_target;

#[derive(Debug, Clone, Serialize)]
pub struct ScanEntry {
    pub display: String,
    pub path: PathBuf,
    pub kind: CleanupKind,
    pub risk: RiskLevel,
    pub physical_bytes: u64,
    pub native: Option<NativeSpec>,
    #[serde(default)]
    pub companions: Vec<PathBuf>,
    #[serde(default)]
    pub shared_bytes: u64,
    #[serde(default)]
    pub size_unknown: bool,
    #[serde(default)]
    pub partial: bool,
    #[serde(default)]
    pub needs_root: bool,
    #[serde(default)]
    pub permanent_only: bool,
}

impl ScanEntry {
    pub fn guard<'a, F: FsOps>(
        &self,
        fs: &'a F,
        config: &'a Config,
        home: &'a Path,
        elevated: bool,
    ) -> Guard<'a, F> {
        Guard::new(fs, config, home).allowing_root_owned(self.needs_root && elevated)
    }

    pub fn size_label(&self) -> String {
        if self.size_unknown {
            return "—".into();
        }
        crate::format::human_bytes(self.physical_bytes)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanCategory {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub entries: Vec<ScanEntry>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub categories: Vec<ScanCategory>,
    pub total_bytes: u64,
}

pub struct ScanCtx<'a, F: FsOps, P: ProcessProbe> {
    pub fs: &'a F,
    pub probe: &'a P,
    pub config: &'a Config,
    pub home: &'a Path,
    pub elevated: bool,
}

impl<F: FsOps, P: ProcessProbe> ScanCtx<'_, F, P> {
    pub fn guard(&self) -> Guard<'_, F> {
        Guard::new(self.fs, self.config, self.home)
    }

    pub fn guard_for(&self, target: &Target) -> Guard<'_, F> {
        self.guard()
            .allowing_root_owned(target.needs_root && self.elevated)
    }
}

pub fn scan<F, P, N>(ctx: &ScanCtx<'_, F, P>, categories: &[Category], native_size: N) -> ScanResult
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
{
    scan_with_progress(ctx, categories, native_size, |_, _, _, _| {})
}

pub fn scan_with_progress<F, P, N, G>(
    ctx: &ScanCtx<'_, F, P>,
    categories: &[Category],
    native_size: N,
    mut on_target: G,
) -> ScanResult
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
    G: FnMut(&str, usize, usize, u64),
{
    let total_targets: usize = categories.iter().map(|c| c.targets.len()).sum();
    let mut out = Vec::new();
    let mut grand_total = 0u64;
    let mut processed = 0usize;
    let mut seen_dirs = std::collections::HashSet::new();
    for cat in categories {
        let mut entries = Vec::new();
        for target in &cat.targets {
            for entry in process_target(ctx, target, &native_size, &mut seen_dirs) {
                grand_total += reclaimable_bytes(&entry);
                entries.push(entry);
            }
            processed += 1;
            on_target(&target.path, processed, total_targets, grand_total);
        }
        let total: u64 = entries.iter().map(reclaimable_bytes).sum();
        out.push(ScanCategory {
            id: cat.id.to_string(),
            name: cat.name.to_string(),
            glyph: cat.glyph.to_string(),
            entries,
            total_bytes: total,
        });
    }
    ScanResult {
        categories: out,
        total_bytes: grand_total,
    }
}

fn reclaimable_bytes(entry: &ScanEntry) -> u64 {
    if entry.kind == CleanupKind::InfoOnly || entry.risk == RiskLevel::Never {
        0
    } else {
        entry.physical_bytes
    }
}
