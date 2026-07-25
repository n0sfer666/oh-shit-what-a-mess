use anyhow::{Context, Result};
use oswam_core::category::{builtin_categories, CleanupKind, NativeSpec};
use oswam_core::config::{default_config_path, Config};
use oswam_core::docker;
use oswam_core::fsops::{FsOps, RealFs};
use oswam_core::guard::Guard;
use oswam_core::privilege::is_root;
use oswam_core::process::LsofProbe;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{scan, ScanCategory, ScanCtx, ScanEntry, ScanResult};
use oswam_core::size::{measure_contents, Measure};
use oswam_core::snapshots;
use std::path::{Path, PathBuf};

pub struct Env {
    pub config: Config,
    pub home: PathBuf,
    pub first_run: bool,
}

pub fn load_env() -> Result<Env> {
    let home = oswam_core::sudo::user_home().context("не удалось определить домашний каталог")?;
    let cfg_path = default_config_path();
    let first_run = cfg_path.as_ref().map(|p| !p.exists()).unwrap_or(true);
    let config = match &cfg_path {
        Some(p) => Config::load(p).context("ошибка чтения config")?,
        None => Config::default(),
    };
    let config = match RealFs::new() {
        Ok(fs) => config.resolved(&fs, &home),
        Err(_) => config,
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
        elevated: is_root(),
    };
    let mut result = scan(&ctx, &builtin_categories(), docker::estimate);
    if is_root() {
        for cat in [
            snapshot_category(),
            system_caches_category(&fs, &env.config, &env.home, is_root()),
        ]
        .into_iter()
        .flatten()
        {
            result.total_bytes += cat.total_bytes;
            result.categories.push(cat);
        }
    }
    Ok(result)
}

pub fn system_caches_category(
    fs: &RealFs,
    config: &Config,
    home: &Path,
    elevated: bool,
) -> Option<ScanCategory> {
    if !elevated {
        return None;
    }
    let root = Path::new("/Library/Caches");
    let children = fs.read_dir(root).ok()?;
    let guard = Guard::new(fs, config, home).allowing_root_owned(true);
    let mut entries = Vec::new();
    let mut total = 0u64;
    for child in children {
        let display = child
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| child.to_string_lossy().into_owned());
        let refused = !guard.allows(&child);
        let m = measure_contents(fs, &child, &|p, meta| guard.allows_meta(p, meta))
            .unwrap_or_else(|_| Measure::unknown());
        if !refused {
            total += m.bytes;
        }
        entries.push(ScanEntry {
            display,
            path: child,
            kind: if refused {
                CleanupKind::InfoOnly
            } else {
                CleanupKind::DeleteContents
            },
            risk: if refused {
                RiskLevel::Never
            } else {
                RiskLevel::Danger
            },
            physical_bytes: m.bytes,
            native: None,
            companions: Vec::new(),
            shared_bytes: m.shared_bytes,
            size_unknown: m.is_unknown(),
            partial: m.partial,
            needs_root: true,
            permanent_only: true,
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
                privileged: true,
                ..NativeSpec::default()
            }),
            companions: Vec::new(),
            shared_bytes: 0,
            size_unknown: true,
            partial: false,
            needs_root: true,
            permanent_only: true,
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
            companions: Vec::new(),
            shared_bytes: 0,
            size_unknown: true,
            partial: false,
            needs_root: true,
            permanent_only: true,
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
