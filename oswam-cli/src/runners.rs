use oswam_core::blocking::Recheck;
use oswam_core::category::CleanupKind;
use oswam_core::delete::{purge_trashed, DeleteReport, Deleter, Disposition};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::guard::Guard;
use oswam_core::native;
use oswam_core::privilege::is_root;
use oswam_tui::app::Summary;
use oswam_tui::run::{DeleteMsg, DeleteRunner, PurgeRunner};

use crate::context::Env;

pub fn build_delete_runner(env: &Env) -> DeleteRunner {
    let config = env.config.clone();
    let home = env.home.clone();
    Box::new(move |entries, disposition, tx| {
        let total = entries.len();
        let Ok(fs) = RealFs::new() else {
            let _ = tx.send(DeleteMsg::Done(failed_all(total), Vec::new()));
            return;
        };
        let elevated = is_root();
        let deleter = Deleter::new(disposition, false);
        let recheck = Recheck::before_delete(&entries);
        let mut summary = Summary::default();
        let mut trashed = Vec::new();
        for (i, entry) in entries.iter().enumerate() {
            let _ = tx.send(DeleteMsg::Progress {
                message: entry.display.clone(),
                done: i,
                total,
                freed: summary.freed + summary.trashed,
            });
            if recheck.blocker(entry).is_some() {
                summary.blocked += 1;
                continue;
            }
            if entry.kind == CleanupKind::NativeCommand {
                match entry.native.as_ref().map(native::run_spec) {
                    Some(Err(_)) => summary.failed += 1,
                    Some(Ok(out)) => {
                        summary.freed +=
                            docker::parse_reclaimed(&out).unwrap_or(entry.physical_bytes);
                        summary.count += 1;
                    }
                    None => summary.count += 1,
                }
                continue;
            }
            let guard = entry.guard(&fs, &config, &home, elevated);
            match deleter.remove(&guard, &entry.path, &entry.companions, entry.kind) {
                Ok(report) => {
                    tally(&mut summary, &report);
                    trashed.extend(report.trashed);
                }
                Err(_) => summary.failed += 1,
            }
        }
        if elevated {
            trashed.clear();
        }
        let _ = tx.send(DeleteMsg::Done(summary, trashed));
    })
}

pub fn build_purge_runner(env: &Env) -> PurgeRunner {
    let config = env.config.clone();
    let home = env.home.clone();
    Box::new(move |items, tx| {
        let total = items.len();
        let Ok(fs) = RealFs::new().and_then(unprivileged) else {
            let _ = tx.send(DeleteMsg::Purged(failed_all(total)));
            return;
        };
        let guard = Guard::new(&fs, &config, &home);
        let mut summary = Summary::default();
        for (i, item) in items.iter().enumerate() {
            let _ = tx.send(DeleteMsg::Progress {
                message: item.path.display().to_string(),
                done: i,
                total,
                freed: summary.freed,
            });
            tally(
                &mut summary,
                &purge_trashed(&guard, std::slice::from_ref(item)),
            );
        }
        let _ = tx.send(DeleteMsg::Purged(summary));
    })
}

fn tally(summary: &mut Summary, report: &DeleteReport) {
    summary.failed += report.failed.len();
    summary.count += report.done.iter().filter(|i| !i.untouched).count();
    summary.untouched += report.done.iter().filter(|i| i.untouched).count();
    summary.partial += report.done.iter().filter(|i| i.partial).count();
    summary.freed += report.bytes(Disposition::Permanent);
    summary.trashed += report.bytes(Disposition::Trash);
}

fn unprivileged(fs: RealFs) -> std::io::Result<RealFs> {
    if is_root() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "purge runs without root",
        ));
    }
    Ok(fs)
}

fn failed_all(total: usize) -> Summary {
    Summary {
        failed: total,
        ..Summary::default()
    }
}
