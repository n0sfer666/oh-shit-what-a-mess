use super::super::{DeleteFailure, Disposition};
use crate::fsops::{FsOps, Meta};
use crate::guard::Guard;
use crate::size::own_bytes;
use std::io;
use std::path::{Path, PathBuf};

pub(super) enum Plan {
    Leaf(u64),
    Dir(Box<DirPlan>),
    Blocked(io::Error),
}

pub(super) struct DirPlan {
    pub own: u64,
    pub total: u64,
    pub whole: bool,
    pub kept: bool,
    pub children: Vec<(PathBuf, Plan)>,
    pub failures: Vec<DeleteFailure>,
}

#[derive(Default)]
struct Children {
    allowed: Vec<(PathBuf, Meta)>,
    kept: bool,
    failures: Vec<DeleteFailure>,
}

pub(super) fn plan_for<F: FsOps>(
    guard: &Guard<'_, F>,
    path: &Path,
    disposition: Disposition,
) -> Plan {
    match guard.fs.meta(path) {
        Ok(meta) => plan(guard, path, &meta, disposition),
        Err(error) => Plan::Blocked(error),
    }
}

fn children_of<F: FsOps>(guard: &Guard<'_, F>, path: &Path) -> io::Result<Children> {
    let mut found = Children::default();
    for child in guard.fs.read_dir(path)? {
        match guard.fs.meta(&child) {
            Ok(meta) if guard.allows_meta(&child, &meta) => found.allowed.push((child, meta)),
            Ok(_) => found.kept = true,
            Err(error) => {
                found.kept = true;
                found.failures.push(DeleteFailure { path: child, error });
            }
        }
    }
    Ok(found)
}

fn plan<F: FsOps>(
    guard: &Guard<'_, F>,
    path: &Path,
    meta: &Meta,
    disposition: Disposition,
) -> Plan {
    if !meta.is_dir || meta.is_symlink {
        return Plan::Leaf(own_bytes(meta));
    }
    let children = match children_of(guard, path) {
        Ok(found) => found,
        Err(error) => return Plan::Blocked(error),
    };
    let own = own_bytes(meta);
    let mut dir = DirPlan {
        own,
        total: own,
        whole: !children.kept,
        kept: children.kept,
        children: Vec::with_capacity(children.allowed.len()),
        failures: children.failures,
    };
    for (child, child_meta) in children.allowed {
        let child_plan = plan(guard, &child, &child_meta, disposition);
        match &child_plan {
            Plan::Leaf(bytes) => dir.total += bytes,
            Plan::Dir(sub) if sub.whole => dir.total += sub.total,
            _ => dir.whole = false,
        }
        dir.children.push((child, child_plan));
    }
    if dir.whole && disposition == Disposition::Trash {
        dir.children = Vec::new();
    }
    Plan::Dir(Box::new(dir))
}
