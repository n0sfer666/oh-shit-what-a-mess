use anyhow::Result;
use oswam_core::category::CleanupKind;
use oswam_core::config::Config;
use oswam_core::delete::{Deleter, Disposition};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::manifest::{now_rfc3339, Manifest};
use oswam_core::scan::ScanEntry;
use std::path::Path;

pub struct Runner<'a> {
    pub fs: &'a RealFs,
    pub config: &'a Config,
    pub home: &'a Path,
    pub elevated: bool,
}

#[derive(Default)]
pub struct Outcome {
    pub manifest: Manifest,
    pub failures: usize,
}

pub fn execute(
    runner: &Runner<'_>,
    entries: &[ScanEntry],
    disposition: Disposition,
    dry_run: bool,
) -> Result<Outcome> {
    let mut outcome = Outcome::default();
    let deleter = Deleter::new(disposition, dry_run);
    for entry in entries {
        if entry.kind == CleanupKind::NativeCommand {
            if let Err(err) = execute_native(entry, dry_run, &mut outcome.manifest) {
                eprintln!("  не удалось {}: {err}", entry.display);
                outcome.failures += 1;
            }
            continue;
        }
        let guard = entry.guard(runner.fs, runner.config, runner.home, runner.elevated);
        let report = match deleter.remove(&guard, &entry.path, &entry.companions, entry.kind) {
            Ok(report) => report,
            Err(err) => {
                eprintln!("  пропущено {}: {err}", entry.display);
                outcome.failures += 1;
                continue;
            }
        };
        for failure in &report.failed {
            eprintln!("  не удалось {}: {}", failure.path.display(), failure.error);
        }
        outcome.failures += report.failed.len();
        for item in report.done {
            if item.untouched {
                eprintln!(
                    "  не тронуто {}: всё содержимое защищено или недоступно",
                    item.path.display()
                );
                continue;
            }
            if item.partial {
                eprintln!(
                    "  частично {}: внутри осталось защищённое",
                    item.path.display()
                );
            }
            outcome.manifest.record(
                &item.path,
                item.physical_bytes,
                item.disposition.action_label(),
                &now_rfc3339(),
            );
        }
    }
    Ok(outcome)
}

fn execute_native(entry: &ScanEntry, dry_run: bool, manifest: &mut Manifest) -> Result<()> {
    let Some(spec) = &entry.native else {
        return Ok(());
    };
    let freed = if dry_run {
        println!("  [dry-run] выполнил бы: {}", spec.clean.join(" "));
        entry.physical_bytes
    } else {
        let out = docker::run_clean(spec)?;
        if !out.trim().is_empty() {
            println!("  {}", out.trim());
        }
        docker::parse_reclaimed(&out).unwrap_or(entry.physical_bytes)
    };
    manifest.record(Path::new(&entry.display), freed, "native", &now_rfc3339());
    Ok(())
}
