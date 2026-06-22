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
    let mut out = Vec::new();
    let mut grand_total = 0u64;
    for cat in categories {
        let mut entries = Vec::new();
        for target in &cat.targets {
            if let Some(spec) = &target.native {
                if let Some(bytes) = native_size(spec) {
                    entries.push(ScanEntry {
                        display: target.path.clone(),
                        path: PathBuf::from(&target.path),
                        kind: target.kind,
                        risk: target.risk,
                        physical_bytes: bytes,
                        native: Some(spec.clone()),
                    });
                }
                continue;
            }
            let expanded = expand_tilde(&target.path, ctx.home);
            if ctx.config.is_ignored(&expanded) {
                continue;
            }
            let Ok(meta) = ctx.fs.meta(&expanded) else {
                continue;
            };
            let facts = facts_from_meta(
                &expanded,
                &meta,
                ctx.config.is_protected(&expanded, ctx.home),
                ctx.probe.holds(&expanded),
            );
            let risk = if target.kind == CleanupKind::InfoOnly {
                target.risk.max(RiskLevel::Caution)
            } else {
                classify(&facts, target.risk)
            };
            let physical_bytes = physical_size(ctx.fs, &expanded).unwrap_or(0);
            entries.push(ScanEntry {
                display: target.path.clone(),
                path: expanded,
                kind: target.kind,
                risk,
                physical_bytes,
                native: None,
            });
        }
        let total: u64 = entries.iter().map(|e| e.physical_bytes).sum();
        grand_total += total;
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
            }],
        }];
        let res = scan(&ctx, &cats, |_| None);
        let entry = &res.categories[0].entries[0];
        assert_eq!(entry.physical_bytes, (8 + 10) * 512);
        assert_eq!(entry.risk, RiskLevel::Safe);
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
            }],
        }];
        let res = scan(&ctx, &cats, |_| None);
        assert_eq!(res.categories[0].entries[0].risk, RiskLevel::Caution);
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
