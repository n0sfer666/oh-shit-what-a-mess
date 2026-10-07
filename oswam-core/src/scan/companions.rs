use super::{ScanCtx, ScanEntry};
use crate::category::{CleanupKind, Target};
use crate::fsops::FsOps;
use crate::process::ProcessProbe;
use crate::risk::RiskLevel;
use crate::size::{measure_where, Measure};
use std::path::{Path, PathBuf};

pub(super) fn sibling_companions<F, P>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    root: &Path,
) -> Vec<PathBuf>
where
    F: FsOps,
    P: ProcessProbe,
{
    let Some(parent) = root.parent() else {
        return Vec::new();
    };
    target
        .companions
        .iter()
        .map(|name| parent.join(name))
        .filter(|extra| ctx.fs.meta(extra).is_ok())
        .collect()
}

pub(super) fn attach<F, P>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    mut entry: ScanEntry,
    extras: Vec<PathBuf>,
) -> ScanEntry
where
    F: FsOps,
    P: ProcessProbe,
{
    let guard = ctx.guard_for(target);
    let (allowed, refused): (Vec<PathBuf>, Vec<PathBuf>) =
        extras.into_iter().partition(|extra| guard.allows(extra));
    entry.companions = allowed;
    if !refused.is_empty() {
        entry.kind = CleanupKind::InfoOnly;
        entry.risk = entry.risk.max(RiskLevel::Caution);
        entry.blocked = false;
    }
    for extra in &entry.companions {
        let m = measure_where(ctx.fs, extra, &|p, meta| guard.allows_meta(p, meta))
            .unwrap_or_else(|_| Measure::unknown());
        entry.physical_bytes += m.bytes;
        entry.shared_bytes += m.shared_bytes;
        entry.partial |= m.partial;
    }
    entry
}
