use anyhow::Result;
use oswam_core::category::builtin_categories;
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::privilege::is_root;
use oswam_core::process::LsofProbe;
use oswam_core::scan::{scan_with_progress, ScanCtx, ScanEntry, ScanResult};
use oswam_core::select::{blocked, selectable, Selection};
use oswam_tui::app::App;
use oswam_tui::detect::detect_from_env;
use oswam_tui::run::{run, Jobs, ScanJob, ScanMsg};

use crate::cli::disposition;
use crate::context::{run_scan, Env};
use crate::output::{print_blocked, print_scan, print_summary};
use crate::perform::{execute, Runner};
use crate::runners::{build_delete_runner, build_purge_runner};

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
    print_blocked(&blocked(&result, &selection));
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
    let jobs = Jobs {
        scan: build_scan_job(env),
        delete: build_delete_runner(env),
        purge: build_purge_runner(env),
    };
    run(app, jobs)?;
    Ok(())
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
        let _ = tx.send(ScanMsg::Progress {
            message: crate::extras::progress_message().into(),
            done: 0,
            total: 0,
            bytes: result.total_bytes,
        });
        crate::extras::complete(&mut result, &fs, &config, &home);
        let _ = tx.send(ScanMsg::Done(result));
    })
}
