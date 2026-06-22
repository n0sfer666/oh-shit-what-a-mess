use crate::category::{Category, CleanupKind, NativeSpec};
use crate::config::Config;
use crate::facts::facts_from_meta;
use crate::fsops::FsOps;
use crate::paths::expand_tilde;
use crate::process::ProcessProbe;
use crate::risk::{classify, RiskLevel};
use crate::size::physical_size;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ScanEntry {
    pub display: String,
    pub path: PathBuf,
    pub kind: CleanupKind,
    pub risk: RiskLevel,
    pub physical_bytes: u64,
    pub native: Option<NativeSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanCategory {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub entries: Vec<ScanEntry>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub categories: Vec<ScanCategory>,
    pub total_bytes: u64,
}

pub struct ScanCtx<'a, F: FsOps, P: ProcessProbe> {
    pub fs: &'a F,
    pub probe: &'a P,
    pub config: &'a Config,
    pub home: &'a Path,
}

pub fn scan<F, P, N>(ctx: &ScanCtx<'_, F, P>, categories: &[Category], native_size: N) -> ScanResult
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
{
    scan_with_progress(ctx, categories, native_size, |_, _, _, _| {})
}

pub fn scan_with_progress<F, P, N, G>(
    ctx: &ScanCtx<'_, F, P>,
    categories: &[Category],
    native_size: N,
    mut on_target: G,
) -> ScanResult
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
    G: FnMut(&str, usize, usize, u64),
{
    let total_targets: usize = categories.iter().map(|c| c.targets.len()).sum();
    let mut out = Vec::new();
    let mut grand_total = 0u64;
    let mut processed = 0usize;
    for cat in categories {
        let mut entries = Vec::new();
        for target in &cat.targets {
            for entry in process_target(ctx, target, &native_size) {
                grand_total += entry.physical_bytes;
                entries.push(entry);
            }
            processed += 1;
            on_target(&target.path, processed, total_targets, grand_total);
        }
        let total: u64 = entries.iter().map(|e| e.physical_bytes).sum();
        out.push(ScanCategory {
            id: cat.id.to_string(),
            name: cat.name.to_string(),
            glyph: cat.glyph.to_string(),
            entries,
            total_bytes: total,
        });
    }
    ScanResult {
        categories: out,
        total_bytes: grand_total,
    }
}

fn process_target<F, P, N>(
    ctx: &ScanCtx<'_, F, P>,
    target: &crate::category::Target,
    native_size: &N,
) -> Vec<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
    N: Fn(&NativeSpec) -> Option<u64>,
{
    if let Some(spec) = &target.native {
        return native_size(spec)
            .map(|bytes| ScanEntry {
                display: target.path.clone(),
                path: PathBuf::from(&target.path),
                kind: target.kind,
                risk: target.risk,
                physical_bytes: bytes,
                native: Some(spec.clone()),
            })
            .into_iter()
            .collect();
    }
    let root = expand_tilde(&target.path, ctx.home);
    if !target.enumerate {
        return entry_for(ctx, target, &root, &target.path)
            .into_iter()
            .collect();
    }
    let Ok(children) = ctx.fs.read_dir(&root) else {
        return Vec::new();
    };
    children
        .into_iter()
        .filter_map(|child| {
            let display = child
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| child.to_string_lossy().into_owned());
            entry_for(ctx, target, &child, &display)
        })
        .collect()
}

fn entry_for<F, P>(
    ctx: &ScanCtx<'_, F, P>,
    target: &crate::category::Target,
    path: &Path,
    display: &str,
) -> Option<ScanEntry>
where
    F: FsOps,
    P: ProcessProbe,
{
    if ctx.config.is_ignored(path) {
        return None;
    }
    let meta = ctx.fs.meta(path).ok()?;
    let facts = facts_from_meta(
        path,
        &meta,
        ctx.config.is_protected(path, ctx.home),
        ctx.probe.holds(path),
    );
    let risk = if target.kind == CleanupKind::InfoOnly {
        target.risk.max(RiskLevel::Caution)
    } else {
        classify(&facts, target.risk)
    };
    let physical_bytes = physical_size(ctx.fs, path).unwrap_or(0);
    Some(ScanEntry {
        display: display.to_string(),
        path: path.to_path_buf(),
        kind: target.kind,
        risk,
        physical_bytes,
        native: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::{builtin_categories, Target};
    use crate::fsops::fake::FakeFs;
    use crate::process::LsofProbe;

    fn ctx_parts() -> (FakeFs, LsofProbe, Config) {
        (
            FakeFs::default(),
            LsofProbe::from_paths(vec![]),
            Config::default(),
        )
    }

    #[test]
    fn skips_missing_targets() {
        let (fs, probe, config) = ctx_parts();
        let home = Path::new("/Users/n0sfer");
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home,
        };
        let res = scan(&ctx, &builtin_categories(), |_| None);
        assert!(res.categories.iter().all(|c| c.entries.is_empty()));
        assert_eq!(res.total_bytes, 0);
    }

    #[test]
    fn computes_size_and_risk_for_existing() {
        let mut fs = FakeFs::default();
        fs.dir("/Users/n0sfer/.npm", &["/Users/n0sfer/.npm/x"]);
        fs.file("/Users/n0sfer/.npm/x", 10);
        let probe = LsofProbe::from_paths(vec![]);
        let config = Config::default();
        let home = Path::new("/Users/n0sfer");
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home,
        };
        let cats = vec![Category {
            id: "dev",
            name: "Dev",
            glyph: "d",
            targets: vec![Target {
                path: "~/.npm".into(),
                kind: CleanupKind::DeleteContents,
                risk: RiskLevel::Safe,
                native: None,
                enumerate: false,
            }],
        }];
        let res = scan(&ctx, &cats, |_| None);
        let entry = &res.categories[0].entries[0];
        assert_eq!(entry.physical_bytes, (8 + 10) * 512);
        assert_eq!(entry.risk, RiskLevel::Safe);
    }

    #[test]
    fn enumerate_expands_children_into_entries() {
        let mut fs = FakeFs::default();
        fs.dir(
            "/Users/n0sfer/Library/Caches",
            &[
                "/Users/n0sfer/Library/Caches/Homebrew",
                "/Users/n0sfer/Library/Caches/Yarn",
            ],
        );
        fs.dir("/Users/n0sfer/Library/Caches/Homebrew", &[]);
        fs.dir("/Users/n0sfer/Library/Caches/Yarn", &[]);
        let probe = LsofProbe::from_paths(vec![]);
        let config = Config::default();
        let home = Path::new("/Users/n0sfer");
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home,
        };
        let cats = vec![Category {
            id: "system",
            name: "S",
            glyph: "s",
            targets: vec![Target {
                path: "~/Library/Caches".into(),
                kind: CleanupKind::DeleteContents,
                risk: RiskLevel::Safe,
                native: None,
                enumerate: true,
            }],
        }];
        let res = scan(&ctx, &cats, |_| None);
        let displays: Vec<&str> = res.categories[0]
            .entries
            .iter()
            .map(|e| e.display.as_str())
            .collect();
        assert!(displays.contains(&"Homebrew"));
        assert!(displays.contains(&"Yarn"));
        assert_eq!(res.categories[0].entries.len(), 2);
    }

    #[test]
    fn process_holding_bumps_risk_in_scan() {
        let mut fs = FakeFs::default();
        fs.dir("/Users/n0sfer/Library/Caches/Arc", &[]);
        let held = vec![PathBuf::from("/Users/n0sfer/Library/Caches/Arc/db")];
        let probe = LsofProbe::from_paths(held);
        let config = Config::default();
        let home = Path::new("/Users/n0sfer");
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home,
        };
        let cats = vec![Category {
            id: "browsers",
            name: "B",
            glyph: "b",
            targets: vec![Target {
                path: "~/Library/Caches/Arc".into(),
                kind: CleanupKind::DeleteContents,
                risk: RiskLevel::Safe,
                native: None,
                enumerate: false,
            }],
        }];
        let res = scan(&ctx, &cats, |_| None);
        assert_eq!(res.categories[0].entries[0].risk, RiskLevel::Caution);
    }

    #[test]
    fn progress_reaches_total_targets() {
        let (fs, probe, config) = ctx_parts();
        let home = Path::new("/Users/n0sfer");
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home,
        };
        let cats = builtin_categories();
        let total_targets: usize = cats.iter().map(|c| c.targets.len()).sum();
        let mut last_done = 0;
        let mut last_total = 0;
        scan_with_progress(
            &ctx,
            &cats,
            |_| None,
            |_label, done, total, _bytes| {
                last_done = done;
                last_total = total;
            },
        );
        assert_eq!(last_total, total_targets);
        assert_eq!(last_done, total_targets);
    }

    #[test]
    fn native_uses_estimator_and_serializes() {
        let (fs, probe, config) = ctx_parts();
        let home = Path::new("/Users/n0sfer");
        let ctx = ScanCtx {
            fs: &fs,
            probe: &probe,
            config: &config,
            home,
        };
        let res = scan(&ctx, &builtin_categories(), |spec| {
            if spec.clean.first().map(String::as_str) == Some("docker") {
                Some(25 * 1024)
            } else {
                None
            }
        });
        let dev = res.categories.iter().find(|c| c.id == "dev").unwrap();
        let docker = dev
            .entries
            .iter()
            .find(|e| e.kind == CleanupKind::NativeCommand)
            .unwrap();
        assert_eq!(docker.physical_bytes, 25 * 1024);
        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("\"risk\":\"caution\""));
    }
}
