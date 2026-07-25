use anyhow::Result;
use oswam_core::category::{builtin_categories, CleanupKind};
use oswam_core::delete::{Deleter, Disposition};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::native;
use oswam_core::privilege::is_root;
use oswam_core::process::LsofProbe;
use oswam_core::scan::{scan_with_progress, ScanCtx, ScanEntry, ScanResult};
use oswam_core::select::{selectable, Selection};
use oswam_tui::app::{App, Summary};
use oswam_tui::detect::detect_from_env;
use oswam_tui::run::{run, DeleteMsg, DeleteRunner, ScanJob, ScanMsg};

use crate::cli::disposition;
use crate::context::{run_scan, snapshot_category, system_caches_category, Env};
use crate::output::{print_scan, print_summary};
use crate::perform::{execute, Runner};

pub fn cmd_scan(env: &Env, json: bool) -> Result<()> {
    let result = run_scan(env)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_scan(&result);
        crate::output::print_tips(is_root());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_clean(
    env: &Env,
    safe: bool,
    categories: Vec<String>,
    dry_run: bool,
    trash: bool,
    delete: bool,
    yes: bool,
) -> Result<()> {
    let result = run_scan(env)?;
    let selection = Selection {
        safe_only: safe,
        categories: (!categories.is_empty()).then_some(categories),
    };
    let chosen: Vec<ScanEntry> = selectable(&result, &selection)
        .into_iter()
        .cloned()
        .collect();
    if chosen.is_empty() {
        println!("Нечего удалять по заданным фильтрам.");
        return Ok(());
    }
    let disp = disposition(trash, delete);
    if !dry_run && !yes {
        println!(
            "Будет затронуто {} элементов. Добавьте --yes для подтверждения или --dry-run для превью.",
            chosen.len()
        );
        return Ok(());
    }
    let fs = RealFs::new()?;
    let runner = Runner {
        fs: &fs,
        config: &env.config,
        home: &env.home,
        elevated: is_root(),
    };
    let outcome = execute(&runner, &chosen, disp, dry_run)?;
    print_summary(&outcome.manifest, dry_run, outcome.failures);
    Ok(())
}

pub fn cmd_tui(env: &Env) -> Result<()> {
    let theme = env.config.theme.unwrap_or_else(detect_from_env);
    let app = App::new(theme, env.first_run, is_root());
    run(app, build_scan_job(env), build_delete_runner(env))?;
    Ok(())
}

fn build_delete_runner(env: &Env) -> DeleteRunner {
    let config = env.config.clone();
    let home = env.home.clone();
    Box::new(move |entries, disposition, tx| {
        let total = entries.len();
        let Ok(fs) = RealFs::new() else {
            let _ = tx.send(DeleteMsg::Done(Summary {
                count: 0,
                freed: 0,
                trashed: 0,
                failed: total,
                partial: 0,
                untouched: 0,
            }));
            return;
        };
        let elevated = is_root();
        let deleter = Deleter::new(disposition, false);
        let mut summary = Summary::default();
        for (i, entry) in entries.iter().enumerate() {
            let _ = tx.send(DeleteMsg::Progress {
                message: entry.display.clone(),
                done: i,
                total,
                freed: summary.freed + summary.trashed,
            });
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
                    summary.failed += report.failed.len();
                    summary.count += report.done.iter().filter(|i| !i.untouched).count();
                    summary.untouched += report.done.iter().filter(|i| i.untouched).count();
                    summary.partial += report.done.iter().filter(|i| i.partial).count();
                    summary.freed += report.bytes(Disposition::Permanent);
                    summary.trashed += report.bytes(Disposition::Trash);
                }
                Err(_) => summary.failed += 1,
            }
        }
        let _ = tx.send(DeleteMsg::Done(summary));
    })
}

fn build_scan_job(env: &Env) -> ScanJob {
    let config = env.config.clone();
    let home = env.home.clone();
    Box::new(move |tx| {
        let _ = tx.send(ScanMsg::Progress {
            message: "Анализ запущенных процессов…".into(),
            done: 0,
            total: 0,
            bytes: 0,
        });
        let probe = LsofProbe::collect();
        let Ok(fs) = RealFs::new() else {
            let _ = tx.send(ScanMsg::Done(ScanResult {
                categories: Vec::new(),
                total_bytes: 0,
            }));
            return;
        };
        let categories = builtin_categories();
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home: &home,
            elevated: oswam_core::privilege::is_root(),
        };
        let mut result = scan_with_progress(
            &ctx,
            &categories,
            docker::estimate,
            |label, done, total, bytes| {
                let _ = tx.send(ScanMsg::Progress {
                    message: format!("Сканирую {label}"),
                    done,
                    total,
                    bytes,
                });
            },
        );
        if is_root() {
            let _ = tx.send(ScanMsg::Progress {
                message: "Снимки Time Machine и системные кэши…".into(),
                done: 0,
                total: 0,
                bytes: result.total_bytes,
            });
            for cat in [
                snapshot_category(),
                system_caches_category(&fs, &config, &home, is_root()),
            ]
            .into_iter()
            .flatten()
            {
                result.total_bytes += cat.total_bytes;
                result.categories.push(cat);
            }
        }
        let _ = tx.send(ScanMsg::Done(result));
    })
}
