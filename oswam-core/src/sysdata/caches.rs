use crate::category::CleanupKind;
use crate::config::Config;
use crate::fsops::FsOps;
use crate::guard::Guard;
use crate::risk::RiskLevel;
use crate::scan::ScanEntry;
use crate::size::{measure_contents, Measure};
use std::path::Path;

pub const ROOT_CACHES: &str = "/Library/Caches";

pub fn root_caches<F: FsOps>(fs: &F, config: &Config, home: &Path) -> Vec<ScanEntry> {
    let root = Path::new(ROOT_CACHES);
    let Ok(children) = fs.read_dir(root) else {
        return Vec::new();
    };
    let guard = Guard::new(fs, config, home).allowing_root_owned(true);
    children
        .into_iter()
        .map(|child| {
            let refused = !guard.allows(&child);
            let m = measure_contents(fs, &child, &|p, meta| guard.allows_meta(p, meta))
                .unwrap_or_else(|_| Measure::unknown());
            ScanEntry {
                display: child.to_string_lossy().into_owned(),
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
                blocking_app: None,
                blocked: false,
            }
        })
        .collect()
}
