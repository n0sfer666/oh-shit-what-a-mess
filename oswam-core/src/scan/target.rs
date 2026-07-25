use super::entry::{child_display, entry_for, root_label};
use super::{ScanCtx, ScanEntry};
use crate::category::{NativeSpec, Target};
use crate::fsops::FsOps;
use crate::paths::expand_tilde;
use crate::process::ProcessProbe;
use crate::size::{measure_where, Measure};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn process_target<F, P, N>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    native_size: &N,
    seen_dirs: &mut HashSet<PathBuf>,
) -> Vec<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
{
    if target.needs_root && !ctx.elevated {
        return Vec::new();
    }
    let root = expand_tilde(&target.path, ctx.home);
    if let Some(spec) = &target.native {
        if target.measured {
            return measured_native(ctx, target, spec, &root, native_size)
                .into_iter()
                .collect();
        }
        return native_entry(target, spec, native_size)
            .into_iter()
            .collect();
    }
    if let Some(names) = &target.discover {
        return discovered_entries(ctx, target, &root, names, seen_dirs);
    }
    if !target.enumerate {
        return entry_for(ctx, target, &root, &target.path)
            .into_iter()
            .collect();
    }
    let Ok(children) = ctx.fs.read_dir(&canonical(ctx.fs, &root)) else {
        return Vec::new();
    };
    if target.group_by_stem {
        return grouped_entries(ctx, target, &children);
    }
    children
        .into_iter()
        .filter_map(|child| {
            let display = child_display(&child);
            entry_for(ctx, target, &child, &display)
        })
        .collect()
}

fn canonical<F: FsOps>(fs: &F, path: &Path) -> PathBuf {
    fs.canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn measured_native<F, P, N>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    spec: &NativeSpec,
    root: &Path,
    native_size: &N,
) -> Option<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
{
    native_size(spec)?;
    if ctx.config.is_ignored(root) {
        return None;
    }
    let Some(mut entry) = entry_for(ctx, target, root, &target.path) else {
        return native_entry(target, spec, native_size);
    };
    entry.native = Some(spec.clone());
    entry.permanent_only = true;
    Some(entry)
}

fn native_entry<N>(target: &Target, spec: &NativeSpec, native_size: &N) -> Option<ScanEntry>
where
    N: Fn(&NativeSpec) -> Option<u64>,
{
    native_size(spec).map(|bytes| ScanEntry {
        display: target.path.clone(),
        path: PathBuf::from(&target.path),
        kind: target.kind,
        risk: target.risk,
        physical_bytes: bytes,
        native: Some(spec.clone()),
        companions: Vec::new(),
        shared_bytes: 0,
        size_unknown: spec.estimate.is_empty(),
        partial: false,
        needs_root: target.needs_root,
        permanent_only: true,
    })
}

fn discovered_entries<F, P>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    root: &Path,
    names: &[&'static str],
    seen_dirs: &mut HashSet<PathBuf>,
) -> Vec<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
{
    let real_root = canonical(ctx.fs, root);
    let label = root_label(root, ctx.home);
    crate::discover::find_named_dirs(ctx.fs, &real_root, names, target.depth)
        .into_iter()
        .filter(|dir| seen_dirs.insert(dir.clone()))
        .filter_map(|dir| {
            let rel = dir
                .strip_prefix(&real_root)
                .unwrap_or(&dir)
                .to_string_lossy();
            entry_for(ctx, target, &dir, &format!("{label}/{rel}"))
        })
        .collect()
}

fn grouped_entries<F, P>(
    ctx: &ScanCtx<'_, F, P>,
    target: &Target,
    children: &[PathBuf],
) -> Vec<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
{
    let guard = ctx.guard_for(target);
    let is_dir = |p: &Path| ctx.fs.meta(p).map(|m| m.is_dir).unwrap_or(false);
    crate::grouping::group_by_stem(children, crate::grouping::GROUPED_EXTENSIONS, is_dir)
        .into_iter()
        .filter_map(|group| {
            let display = child_display(&group.primary);
            let mut entry = entry_for(ctx, target, &group.primary, &display)?;
            let (allowed, refused): (Vec<PathBuf>, Vec<PathBuf>) = group
                .companions
                .into_iter()
                .partition(|extra| guard.allows(extra));
            entry.companions = allowed;
            if !refused.is_empty() {
                entry.kind = crate::category::CleanupKind::InfoOnly;
                entry.risk = entry.risk.max(crate::risk::RiskLevel::Caution);
            }
            for extra in &entry.companions {
                let m = measure_where(ctx.fs, extra, &|p, meta| guard.allows_meta(p, meta))
                    .unwrap_or_else(|_| Measure::unknown());
                entry.physical_bytes += m.bytes;
                entry.shared_bytes += m.shared_bytes;
                entry.partial |= m.partial;
            }
            Some(entry)
        })
        .collect()
}
