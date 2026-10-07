use super::{DeleteFailure, DeleteReport, DeletedItem, Deleter, Disposition, Trashed};
use crate::category::CleanupKind;
use crate::fsops::{trash_root, FsOps};
use crate::guard::Guard;

pub fn purge_trashed<F: FsOps>(guard: &Guard<'_, F>, items: &[Trashed]) -> DeleteReport {
    let deleter = Deleter::new(Disposition::Permanent, false);
    let mut report = DeleteReport::default();
    for item in items {
        if !still_ours(guard, item) {
            report.done.push(DeletedItem {
                path: item.path.clone(),
                physical_bytes: 0,
                disposition: Disposition::Permanent,
                partial: false,
                untouched: true,
            });
            continue;
        }
        match deleter.remove(guard, &item.path, &[], CleanupKind::DeletePath) {
            Ok(done) => report.absorb(done),
            Err(error) => report.failed.push(DeleteFailure {
                path: item.path.clone(),
                error,
            }),
        }
    }
    report
}

fn still_ours<F: FsOps>(guard: &Guard<'_, F>, item: &Trashed) -> bool {
    let canonical = |p: &std::path::Path| guard.fs.canonicalize(p).ok();
    let trash = canonical(&trash_root(guard.home));
    item.path
        .parent()
        .and_then(canonical)
        .is_some_and(|parent| Some(parent) == trash)
        && guard
            .fs
            .meta(&item.path)
            .is_ok_and(|m| m.ino == item.ino && m.dev == item.dev)
}
