use anyhow::{Context, Result};
use oswam_core::category::{builtin_categories, CleanupKind, NativeSpec};
use oswam_core::config::{default_config_path, Config};
use oswam_core::docker;
use oswam_core::fsops::{is_dataless, is_sip_protected, FsOps, RealFs};
use oswam_core::privilege::is_root;
use oswam_core::process::LsofProbe;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{scan, ScanCategory, ScanCtx, ScanEntry, ScanResult};
use oswam_core::size::physical_size;
use oswam_core::snapshots;
use std::path::{Path, PathBuf};

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
        for cat in [snapshot_category(), system_caches_category()]
            .into_iter()
            .flatten()
        {
            result.total_bytes += cat.total_bytes;
            result.categories.push(cat);
        }
    }
    Ok(result)
}

pub fn system_caches_category() -> Option<ScanCategory> {
    let fs = RealFs::new().ok()?;
    let root = Path::new("/Library/Caches");
    let children = fs.read_dir(root).ok()?;
    let mut entries = Vec::new();
    let mut total = 0u64;
    for child in children {
        let Ok(meta) = fs.meta(&child) else {
            continue;
        };
        if is_sip_protected(meta.flags) || is_dataless(meta.flags) {
            continue;
        }
        let display = child
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| child.to_string_lossy().into_owned());
        let physical_bytes = physical_size(&fs, &child).unwrap_or(0);
        total += physical_bytes;
        entries.push(ScanEntry {
            display,
            path: child,
            kind: CleanupKind::DeleteContents,
            risk: RiskLevel::Danger,
            physical_bytes,
            native: None,
        });
    }
    if entries.is_empty() {
        return None;
    }
    Some(ScanCategory {
        id: "system-caches".to_string(),
        name: "Системные кэши /Library/Caches (sudo, риск)".to_string(),
        glyph: "⚙".to_string(),
        entries,
        total_bytes: total,
    })
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
