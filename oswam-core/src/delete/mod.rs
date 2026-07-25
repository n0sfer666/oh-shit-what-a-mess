mod report;
#[cfg(test)]
mod tests;
mod walk;

pub use report::{DeleteFailure, DeleteReport, DeletedItem, Disposition};

use crate::category::CleanupKind;
use crate::fsops::FsOps;
use crate::guard::Guard;
use std::io;
use std::path::{Path, PathBuf};

pub struct Deleter {
    disposition: Disposition,
    dry_run: bool,
}

impl Deleter {
    pub fn new(disposition: Disposition, dry_run: bool) -> Self {
        Self {
            disposition,
            dry_run,
        }
    }

    pub fn remove<F: FsOps>(
        &self,
        guard: &Guard<'_, F>,
        path: &Path,
        companions: &[PathBuf],
        kind: CleanupKind,
    ) -> io::Result<DeleteReport> {
        let mut report = DeleteReport::default();
        if companions.iter().any(|extra| !guard.allows(extra)) {
            report.done.push(self.untouched(guard, path));
            return Ok(report);
        }
        let primaries = primary_items(guard, path, kind)?;
        if primaries.items.is_empty() {
            if primaries.refused {
                report.done.push(self.untouched(guard, path));
            }
            return Ok(report);
        }
        let extras: Vec<PathBuf> = companions.to_vec();
        let planned = walk::plan_all(guard, primaries.items, self.disposition);
        let planned_extras = walk::plan_all(guard, extras, self.disposition);
        let mut kept = false;
        for item in planned {
            kept |= self.take(guard, item, &mut report);
        }
        if report.done.is_empty() || kept {
            return Ok(report);
        }
        for extra in planned_extras {
            self.take(guard, extra, &mut report);
        }
        Ok(report)
    }

    fn untouched<F: FsOps>(&self, guard: &Guard<'_, F>, path: &Path) -> DeletedItem {
        DeletedItem {
            path: path.to_path_buf(),
            physical_bytes: 0,
            disposition: effective(guard.fs, path, self.disposition),
            partial: false,
            untouched: true,
        }
    }

    fn take<F: FsOps>(
        &self,
        guard: &Guard<'_, F>,
        item: walk::Planned,
        report: &mut DeleteReport,
    ) -> bool {
        let (path, disposition) = (item.path.clone(), item.disposition);
        let walked = if self.dry_run {
            walk::preview(item)
        } else {
            walk::apply(guard, item)
        };
        let (kept, touched) = (walked.kept, walked.touched);
        let stalled = !touched && !walked.failures.is_empty();
        report.failed.extend(walked.failures);
        if stalled {
            return true;
        }
        report.done.push(DeletedItem {
            path,
            physical_bytes: walked.freed,
            disposition,
            partial: kept && touched,
            untouched: !touched,
        });
        kept
    }
}

struct Primaries {
    items: Vec<PathBuf>,
    refused: bool,
}

fn primary_items<F: FsOps>(
    guard: &Guard<'_, F>,
    path: &Path,
    kind: CleanupKind,
) -> io::Result<Primaries> {
    match kind {
        CleanupKind::DeleteContents => {
            let meta = guard.fs.meta(path)?;
            if !meta.is_dir || meta.is_symlink {
                return primary_items(guard, path, CleanupKind::DeletePath);
            }
            let children = guard.fs.read_dir(path)?;
            let items: Vec<PathBuf> = children
                .iter()
                .filter(|child| guard.allows(child))
                .cloned()
                .collect();
            let refused = items.len() != children.len();
            Ok(Primaries { items, refused })
        }
        CleanupKind::DeletePath if guard.allows(path) => Ok(Primaries {
            items: vec![path.to_path_buf()],
            refused: false,
        }),
        CleanupKind::DeletePath => Ok(Primaries {
            items: Vec::new(),
            refused: true,
        }),
        CleanupKind::NativeCommand | CleanupKind::InfoOnly => Ok(Primaries {
            items: Vec::new(),
            refused: false,
        }),
    }
}

pub fn effective<F: FsOps>(fs: &F, path: &Path, requested: Disposition) -> Disposition {
    match requested {
        Disposition::Trash if !fs.can_trash(path) => Disposition::Permanent,
        other => other,
    }
}
