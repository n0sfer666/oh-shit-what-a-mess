use anyhow::Result;
use oswam_core::category::{builtin_categories, CleanupKind};
use oswam_core::delete::{delete_target, Disposition};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::process::LsofProbe;
use oswam_core::scan::{scan_with_progress, ScanCtx, ScanEntry, ScanResult};
use oswam_core::select::{selectable, Selection};
use oswam_tui::app::App;
use oswam_tui::detect::detect_from_env;
use oswam_tui::run::{run, DeleteMsg, DeleteRunner, ScanJob, ScanMsg};

use crate::cli::disposition;
use crate::context::{run_scan, Env};
use crate::output::{print_scan, print_summary};
use crate::perform::execute;

pub fn cmd_scan(env: &Env, json: bool) -> Result<()> {
    let result = run_scan(env)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_scan(&result);
        crate::output::print_tips();
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
    let manifest = execute(&fs, &chosen, disp, dry_run)?;
    print_summary(&manifest, dry_run, disp == Disposition::Trash);
    Ok(())
}

pub fn cmd_tui(env: &Env) -> Result<()> {
    let theme = env.config.theme.unwrap_or_else(detect_from_env);
    let app = App::new(theme, env.first_run);
    run(app, build_scan_job(env), build_delete_runner())?;
    Ok(())
}

fn build_delete_runner() -> DeleteRunner {
    Box::new(|entries, disposition, tx| {
        let total = entries.len();
        let trashed = disposition == Disposition::Trash;
        let Ok(fs) = RealFs::new() else {
            let _ = tx.send(DeleteMsg::Done {
                count: 0,
                freed: 0,
                trashed,
            });
            return;
        };
        let mut freed = 0u64;
        let mut count = 0usize;
        for (i, entry) in entries.iter().enumerate() {
            let _ = tx.send(DeleteMsg::Progress {
                message: entry.display.clone(),
                done: i,
                total,
                freed,
            });
            if entry.kind == CleanupKind::NativeCommand {
                if let Some(spec) = &entry.native {
                    let _ = docker::run_clean(spec);
                }
                freed += entry.physical_bytes;
                count += 1;
                continue;
            }
            if let Ok(items) = delete_target(&fs, &entry.path, entry.kind, disposition, false) {
                for item in items {
                    freed += item.physical_bytes;
                    count += 1;
                }
            }
        }
        let _ = tx.send(DeleteMsg::Done {
            count,
            freed,
            trashed,
        });
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
        };
        let result = scan_with_progress(
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
        let _ = tx.send(ScanMsg::Done(result));
    })
}
