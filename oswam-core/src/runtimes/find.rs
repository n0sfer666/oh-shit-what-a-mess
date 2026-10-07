use super::{label, orphan_assets, orphan_entry, registered_images, ASSETS};
use crate::fsops::FsOps;
use crate::guard::Guard;
use crate::scan::ScanEntry;
use std::path::Path;

pub fn find<F, R, T>(fs: &F, guard: &Guard<'_, F>, run: R, read: T) -> Vec<ScanEntry>
where
    F: FsOps,
    R: Fn(&[&str]) -> Option<String>,
    T: Fn(&Path) -> Option<String>,
{
    if run(&["xcode-select", "-p"]).is_none() {
        return Vec::new();
    }
    let Some(registered) = run(&["xcrun", "simctl", "runtime", "list", "-v"])
        .as_deref()
        .and_then(registered_images)
    else {
        return Vec::new();
    };
    orphan_assets(fs, Path::new(ASSETS), &registered)
        .into_iter()
        .filter_map(|asset| {
            let plist = read(&asset.join("Info.plist"));
            let display = label(plist.as_deref(), &asset);
            orphan_entry(fs, guard, &asset, display)
        })
        .collect()
}
