use super::{ScanCtx, ScanEntry};
use crate::category::{CleanupKind, Target};
use crate::facts::facts_from_meta;
use crate::fsops::FsOps;
use crate::process::ProcessProbe;
use crate::risk::{classify, RiskLevel};
use crate::size::{measure_contents, measure_where, Measure};
use std::path::Path;

pub(super) fn entry_for<F, P>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    path: &Path,
    display: &str,
) -> Option<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
{
    if ctx.config.is_ignored(path) {
        return None;
    }
    let meta = ctx.fs.meta(path).ok()?;
    let mut facts = facts_from_meta(
        path,
        &meta,
        ctx.config.is_protected(path, ctx.home),
        ctx.probe.holds(path),
    );
    if target.needs_root && ctx.elevated {
        facts.is_root_owned = false;
    }
    let classified = classify(&facts, target.risk);
    let risk = if target.kind == CleanupKind::InfoOnly || crate::refetch::costs_a_download(path) {
        classified.max(RiskLevel::Caution)
    } else {
        classified
    };
    let guard = ctx.guard_for(target);
    let allowed = |p: &Path, meta: &crate::fsops::Meta| guard.allows_meta(p, meta);
    let m = match target.kind {
        CleanupKind::DeleteContents => measure_contents(ctx.fs, path, &allowed),
        _ => measure_where(ctx.fs, path, &allowed),
    }
    .unwrap_or_else(|_| Measure::unknown());
    Some(ScanEntry {
        display: display.to_string(),
        path: path.to_path_buf(),
        kind: target.kind,
        risk,
        physical_bytes: m.bytes,
        native: None,
        companions: Vec::new(),
        shared_bytes: m.shared_bytes,
        size_unknown: m.is_unknown(),
        partial: m.partial,
        needs_root: target.needs_root,
        permanent_only: !ctx.fs.can_trash(path),
    })
}

pub(super) fn root_label(root: &Path, home: &Path) -> String {
    let rel = root.strip_prefix(home).unwrap_or(root);
    let tail: Vec<String> = rel
        .iter()
        .rev()
        .take(2)
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| s != "/")
        .collect();
    if tail.is_empty() {
        return child_display(root);
    }
    tail.into_iter().rev().collect::<Vec<_>>().join("/")
}

pub(super) fn child_display(child: &Path) -> String {
    child
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| child.to_string_lossy().into_owned())
}
