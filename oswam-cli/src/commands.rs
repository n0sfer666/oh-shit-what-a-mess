use anyhow::Result;
use oswam_core::delete::Disposition;
use oswam_core::fsops::RealFs;
use oswam_core::scan::ScanEntry;
use oswam_core::select::{selectable, Selection};
use oswam_tui::app::App;
use oswam_tui::detect::detect_from_env;
use oswam_tui::run::run;
use std::io::{self, Write};

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
    let result = run_scan(env)?;
    let theme = env.config.theme.unwrap_or_else(detect_from_env);
    let mut app = App::new(result, theme, env.first_run);
    let Some(entries) = run(&mut app)? else {
        return Ok(());
    };
    if entries.is_empty() {
        println!("Ничего не выбрано.");
        return Ok(());
    }
    let Some(disp) = prompt_disposition()? else {
        println!("Отменено.");
        return Ok(());
    };
    let fs = RealFs::new()?;
    let manifest = execute(&fs, &entries, disp, false)?;
    print_summary(&manifest, false, disp == Disposition::Trash);
    Ok(())
}

fn prompt_disposition() -> Result<Option<Disposition>> {
    print!("Удалить в [t]Корзину / [d]безвозвратно / [c]отмена? ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(match line.trim() {
        "t" | "T" => Some(Disposition::Trash),
        "d" | "D" => Some(Disposition::Permanent),
        _ => None,
    })
}
