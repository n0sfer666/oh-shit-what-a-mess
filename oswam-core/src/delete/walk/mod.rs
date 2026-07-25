mod plan;

use super::{DeleteFailure, Disposition};
use crate::fsops::FsOps;
use crate::guard::Guard;
use plan::{plan_for, DirPlan, Plan};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(super) struct Walked {
    pub kept: bool,
    pub freed: u64,
    pub touched: bool,
    pub failures: Vec<DeleteFailure>,
}

impl Walked {
    fn absorb(&mut self, other: Walked) {
        self.kept |= other.kept;
        self.freed += other.freed;
        self.touched |= other.touched;
        self.failures.extend(other.failures);
    }

    fn gone(freed: u64) -> Self {
        Walked {
            freed,
            touched: true,
            ..Walked::default()
        }
    }

    fn failed(path: &Path, error: io::Error) -> Self {
        Walked {
            kept: true,
            failures: vec![DeleteFailure {
                path: path.to_path_buf(),
                error,
            }],
            ..Walked::default()
        }
    }
}

pub(super) struct Planned {
    pub path: PathBuf,
    pub disposition: Disposition,
    plan: Plan,
}

pub(super) fn plan_all<F: FsOps>(
    guard: &Guard<'_, F>,
    items: Vec<PathBuf>,
    requested: Disposition,
) -> Vec<Planned> {
    items
        .into_iter()
        .map(|path| {
            let disposition = super::effective(guard.fs, &path, requested);
            let plan = plan_for(guard, &path, disposition);
            Planned {
                path,
                disposition,
                plan,
            }
        })
        .collect()
}

pub(super) fn apply<F: FsOps>(guard: &Guard<'_, F>, planned: Planned) -> Walked {
    match planned.disposition {
        Disposition::Trash => trash(guard, &planned.path, planned.plan),
        Disposition::Permanent => purge(guard, &planned.path, planned.plan),
    }
}

pub(super) fn preview(planned: Planned) -> Walked {
    foresee(&planned.path, planned.plan)
}

fn trash<F: FsOps>(guard: &Guard<'_, F>, path: &Path, plan: Plan) -> Walked {
    match plan {
        Plan::Blocked(error) => Walked::failed(path, error),
        Plan::Leaf(bytes) => move_out(guard, path, bytes),
        Plan::Dir(dir) if dir.whole => move_out(guard, path, dir.total),
        Plan::Dir(dir) => {
            let dir: DirPlan = *dir;
            let mut walked = Walked {
                kept: true,
                failures: dir.failures,
                ..Walked::default()
            };
            for (child, child_plan) in dir.children {
                walked.absorb(trash(guard, &child, child_plan));
            }
            walked
        }
    }
}

fn move_out<F: FsOps>(guard: &Guard<'_, F>, path: &Path, bytes: u64) -> Walked {
    match guard.fs.move_to_trash(path) {
        Ok(()) => Walked::gone(bytes),
        Err(error) => Walked::failed(path, error),
    }
}

fn purge<F: FsOps>(guard: &Guard<'_, F>, path: &Path, plan: Plan) -> Walked {
    let dir = match plan {
        Plan::Blocked(error) => return Walked::failed(path, error),
        Plan::Leaf(bytes) => {
            return match guard.fs.remove_file(path) {
                Ok(()) => Walked::gone(bytes),
                Err(error) => Walked::failed(path, error),
            }
        }
        Plan::Dir(dir) => *dir,
    };
    let mut walked = Walked {
        kept: dir.kept,
        failures: dir.failures,
        ..Walked::default()
    };
    for (child, child_plan) in dir.children {
        walked.absorb(purge(guard, &child, child_plan));
    }
    if walked.kept {
        return walked;
    }
    match guard.fs.remove_dir(path) {
        Ok(()) => walked.absorb(Walked::gone(dir.own)),
        Err(error) => walked.absorb(Walked::failed(path, error)),
    }
    walked
}

fn foresee(path: &Path, plan: Plan) -> Walked {
    let dir = match plan {
        Plan::Blocked(error) => return Walked::failed(path, error),
        Plan::Leaf(bytes) => return Walked::gone(bytes),
        Plan::Dir(dir) if dir.whole => return Walked::gone(dir.total),
        Plan::Dir(dir) => *dir,
    };
    let mut walked = Walked {
        kept: true,
        failures: dir.failures,
        ..Walked::default()
    };
    for (child, child_plan) in dir.children {
        walked.absorb(foresee(&child, child_plan));
    }
    walked
}
