use super::firmlinked;
use crate::fsops::FsOps;
use std::path::{Path, PathBuf};

pub fn orphan_assets<F: FsOps>(fs: &F, root: &Path, registered: &[PathBuf]) -> Vec<PathBuf> {
    let root = firmlinked(root);
    let mut orphans: Vec<PathBuf> = fs
        .read_dir(&root)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "asset"))
        .filter(|p| fs.meta(p).is_ok_and(|m| m.is_dir && !m.is_symlink))
        .filter(|p| !registered.iter().any(|image| image.starts_with(p)))
        .collect();
    orphans.sort();
    orphans
}
