use crate::context::snapshot_category;
use oswam_core::config::Config;
use oswam_core::fsops::RealFs;
use oswam_core::guard::Guard;
use oswam_core::native;
use oswam_core::privilege::is_root;
use oswam_core::runtimes;
use oswam_core::scan::ScanResult;
use oswam_core::sysdata::{absorb, root_caches};
use std::path::Path;
use std::time::Duration;

const PROBE_LIMIT: Duration = Duration::from_secs(10);

pub fn progress_message() -> &'static str {
    if is_root() {
        "System Data: рантаймы симуляторов, снимки и системные кэши…"
    } else {
        "System Data: рантаймы симуляторов…"
    }
}

pub fn complete(result: &mut ScanResult, fs: &RealFs, config: &Config, home: &Path) {
    let guard = Guard::new(fs, config, home);
    let run = |argv: &[&str]| {
        let argv: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
        native::run_within(&argv, false, PROBE_LIMIT).ok()
    };
    let read = |path: &Path| std::fs::read_to_string(path).ok();
    let mut entries = runtimes::find(fs, &guard, run, read);
    if is_root() {
        entries.extend(root_caches(fs, config, home));
    }
    absorb(result, entries);
    if is_root() {
        if let Some(snaps) = snapshot_category() {
            result.total_bytes += snaps.total_bytes;
            result.categories.push(snaps);
        }
    }
}
