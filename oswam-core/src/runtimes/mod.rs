mod entry;
mod find;
mod listing;
mod orphans;
mod plist;

pub use entry::{label, orphan_entry};
pub use find::find;
pub use listing::registered_images;
pub use orphans::orphan_assets;
pub use plist::plist_string;

use std::path::{Path, PathBuf};

pub const ASSETS: &str = "/System/Library/AssetsV2/com_apple_MobileAsset_iOSSimulatorRuntime";

const DATA_VOLUME: &str = "/System/Volumes/Data";

fn firmlinked(path: &Path) -> PathBuf {
    path.strip_prefix(DATA_VOLUME)
        .map(|rel| Path::new("/").join(rel))
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests;
