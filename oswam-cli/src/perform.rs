use anyhow::Result;
use oswam_core::category::CleanupKind;
use oswam_core::delete::{delete_target, Disposition};
use oswam_core::docker;
use oswam_core::fsops::RealFs;
use oswam_core::manifest::{now_rfc3339, Manifest};
use oswam_core::risk::RiskLevel;
use oswam_core::scan::ScanEntry;

pub fn execute(
    fs: &RealFs,
    entries: &[ScanEntry],
    disposition: Disposition,
    dry_run: bool,
) -> Result<Manifest> {
    let mut manifest = Manifest::default();
    for entry in entries {
        if entry.kind == CleanupKind::NativeCommand {
            execute_native(entry, dry_run, &mut manifest)?;
            continue;
        }
        let disp = if entry.risk == RiskLevel::Danger {
            Disposition::Permanent
        } else {
            disposition
        };
        let items = delete_target(fs, &entry.path, entry.kind, disp, dry_run)?;
        for item in items {
            manifest.record(
                &item.path,
                item.physical_bytes,
                disp.action_label(),
                &now_rfc3339(),
            );
        }
    }
    Ok(manifest)
}

fn execute_native(entry: &ScanEntry, dry_run: bool, manifest: &mut Manifest) -> Result<()> {
    let Some(spec) = &entry.native else {
        return Ok(());
    };
    if dry_run {
        println!("  [dry-run] выполнил бы: {}", spec.clean.join(" "));
    } else {
        let out = docker::run_clean(spec)?;
        if !out.trim().is_empty() {
            println!("  {}", out.trim());
        }
    }
    manifest.record(
        std::path::Path::new(&entry.display),
        entry.physical_bytes,
        "native",
        &now_rfc3339(),
    );
    Ok(())
}
