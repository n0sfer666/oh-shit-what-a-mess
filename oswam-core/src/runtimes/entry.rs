use super::plist_string;
use crate::category::CleanupKind;
use crate::fsops::FsOps;
use crate::guard::Guard;
use crate::risk::RiskLevel;
use crate::scan::ScanEntry;
use crate::size::{measure_where, Measure};
use std::path::Path;

pub fn label(info_plist: Option<&str>, asset: &Path) -> String {
    let field = |key| info_plist.and_then(|xml| plist_string(xml, key));
    match (field("SimulatorVersion"), field("Build")) {
        (Some(version), Some(build)) => format!("iOS {version} ({build}) — осиротевший рантайм"),
        (Some(version), None) => format!("iOS {version} — осиротевший рантайм"),
        _ => format!(
            "{} — осиротевший рантайм",
            asset.file_name().unwrap_or_default().to_string_lossy()
        ),
    }
}

pub fn orphan_entry<F: FsOps>(
    fs: &F,
    guard: &Guard<'_, F>,
    asset: &Path,
    display: String,
) -> Option<ScanEntry> {
    if !guard.admits(asset) {
        return None;
    }
    let m =
        measure_where(fs, asset, &|p, _| guard.admits(p)).unwrap_or_else(|_| Measure::unknown());
    Some(ScanEntry {
        display,
        path: asset.to_path_buf(),
        kind: CleanupKind::InfoOnly,
        risk: RiskLevel::Never,
        physical_bytes: m.bytes,
        native: None,
        companions: Vec::new(),
        shared_bytes: m.shared_bytes,
        size_unknown: m.is_unknown(),
        partial: m.partial,
        needs_root: false,
        permanent_only: true,
        blocking_app: None,
        blocked: false,
    })
}
