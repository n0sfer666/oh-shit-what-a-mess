use anyhow::{Context, Result};
use oswam_core::category::{builtin_categories, CleanupKind, NativeSpec};
use oswam_core::config::{default_config_path, Config};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::privilege::is_root;
use oswam_core::process::LsofProbe;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{scan, ScanCategory, ScanCtx, ScanEntry, ScanResult};
use oswam_core::snapshots;
use std::path::PathBuf;

pub struct Env {
    pub config: Config,
    pub home: PathBuf,
    pub first_run: bool,
}

pub fn load_env() -> Result<Env> {
    let home = dirs::home_dir().context("не удалось определить домашний каталог")?;
    let cfg_path = default_config_path();
    let first_run = cfg_path.as_ref().map(|p| !p.exists()).unwrap_or(true);
    let config = match &cfg_path {
        Some(p) => Config::load(p).context("ошибка чтения config")?,
        None => Config::default(),
    };
    Ok(Env {
        config,
        home,
        first_run,
    })
}

pub fn run_scan(env: &Env) -> Result<ScanResult> {
    let fs = RealFs::new()?;
    let probe = LsofProbe::collect();
    let ctx = ScanCtx {
        fs: &fs,
        probe: &probe,
        config: &env.config,
        home: &env.home,
    };
    let mut result = scan(&ctx, &builtin_categories(), docker::estimate);
    if is_root() {
        if let Some(cat) = snapshot_category() {
            result.categories.push(cat);
        }
    }
    Ok(result)
}

pub fn snapshot_category() -> Option<ScanCategory> {
    let snaps = snapshots::list_local_snapshots();
    if snaps.is_empty() {
        return None;
    }
    let (deletable, latest) = snapshots::split_keep_latest(snaps);
    let mut entries: Vec<ScanEntry> = deletable
        .iter()
        .map(|date| ScanEntry {
            display: date.clone(),
            path: PathBuf::from(date),
            kind: CleanupKind::NativeCommand,
            risk: RiskLevel::Caution,
            physical_bytes: 0,
            native: Some(NativeSpec {
                estimate: Vec::new(),
                clean: snapshots::delete_command(date),
            }),
        })
        .collect();
    if let Some(latest) = latest {
        entries.push(ScanEntry {
            display: format!("{latest} (последний — сохраняется)"),
            path: PathBuf::from(&latest),
            kind: CleanupKind::InfoOnly,
            risk: RiskLevel::Never,
            physical_bytes: 0,
            native: None,
        });
    }
    Some(ScanCategory {
        id: "snapshots".to_string(),
        name: "Снимки Time Machine".to_string(),
        glyph: "🕒".to_string(),
        entries,
        total_bytes: 0,
    })
}
