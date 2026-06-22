use anyhow::{Context, Result};
use oswam_core::category::builtin_categories;
use oswam_core::config::{default_config_path, Config};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::process::LsofProbe;
use oswam_core::scan::{scan, ScanCtx, ScanResult};
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
    Ok(scan(&ctx, &builtin_categories(), docker::estimate))
}
