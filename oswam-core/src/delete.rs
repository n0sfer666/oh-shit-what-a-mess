use crate::category::CleanupKind;
use crate::fsops::FsOps;
use crate::size::physical_size;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Disposition {
    Trash,
    Permanent,
}

impl Disposition {
    pub fn action_label(&self) -> &'static str {
        match self {
            Disposition::Trash => "trash",
            Disposition::Permanent => "permanent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedItem {
    pub path: PathBuf,
    pub physical_bytes: u64,
}

pub fn delete_target<F: FsOps>(
    fs: &F,
    path: &Path,
    kind: CleanupKind,
    disposition: Disposition,
    dry_run: bool,
) -> io::Result<Vec<DeletedItem>> {
    let items = top_level_items(fs, path, kind)?;
    let mut done = Vec::with_capacity(items.len());
    for item in items {
        let physical_bytes = physical_size(fs, &item)?;
        if !dry_run {
            apply(fs, &item, disposition)?;
        }
        done.push(DeletedItem {
            path: item,
            physical_bytes,
        });
    }
    Ok(done)
}

fn top_level_items<F: FsOps>(fs: &F, path: &Path, kind: CleanupKind) -> io::Result<Vec<PathBuf>> {
    match kind {
        CleanupKind::DeleteContents => fs.read_dir(path),
        CleanupKind::DeletePath => Ok(vec![path.to_path_buf()]),
        CleanupKind::NativeCommand | CleanupKind::InfoOnly => Ok(Vec::new()),
    }
}

fn apply<F: FsOps>(fs: &F, path: &Path, disposition: Disposition) -> io::Result<()> {
    match disposition {
        Disposition::Trash => fs.move_to_trash(path),
        Disposition::Permanent => remove_recursive(fs, path),
    }
}

fn remove_recursive<F: FsOps>(fs: &F, path: &Path) -> io::Result<()> {
    let meta = fs.meta(path)?;
    if meta.is_dir && !meta.is_symlink {
        for child in fs.read_dir(path)? {
            remove_recursive(fs, &child)?;
        }
        fs.remove_dir(path)
    } else {
        fs.remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsops::fake::FakeFs;

    fn tree() -> FakeFs {
        let mut fs = FakeFs::default();
        fs.dir("/cache", &["/cache/a", "/cache/b"]);
        fs.file("/cache/a", 4);
        fs.file("/cache/b", 6);
        fs
    }

    #[test]
    fn dry_run_touches_nothing_but_reports_sizes() {
        let fs = tree();
        let items = delete_target(
            &fs,
            Path::new("/cache"),
            CleanupKind::DeleteContents,
            Disposition::Permanent,
            true,
        )
        .unwrap();
        assert_eq!(items.len(), 2);
        assert!(fs.removed.borrow().is_empty());
        assert!(fs.trashed.borrow().is_empty());
        let total: u64 = items.iter().map(|i| i.physical_bytes).sum();
        assert_eq!(total, (4 + 6) * 512);
    }

    #[test]
    fn permanent_removes_children() {
        let fs = tree();
        delete_target(
            &fs,
            Path::new("/cache"),
            CleanupKind::DeleteContents,
            Disposition::Permanent,
            false,
        )
        .unwrap();
        let removed = fs.removed.borrow();
        assert!(removed.contains(&PathBuf::from("/cache/a")));
        assert!(removed.contains(&PathBuf::from("/cache/b")));
        assert!(!removed.contains(&PathBuf::from("/cache")));
    }

    #[test]
    fn trash_moves_children() {
        let fs = tree();
        delete_target(
            &fs,
            Path::new("/cache"),
            CleanupKind::DeleteContents,
            Disposition::Trash,
            false,
        )
        .unwrap();
        assert_eq!(fs.trashed.borrow().len(), 2);
        assert!(fs.removed.borrow().is_empty());
    }

    #[test]
    fn info_only_yields_nothing() {
        let fs = tree();
        let items = delete_target(
            &fs,
            Path::new("/cache"),
            CleanupKind::InfoOnly,
            Disposition::Trash,
            false,
        )
        .unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn symlink_not_followed_on_remove() {
        let mut fs = FakeFs::default();
        fs.dir("/d", &["/d/link"]);
        fs.entries.insert(
            PathBuf::from("/d/link"),
            crate::fsops::Meta {
                is_symlink: true,
                blocks: 0,
                ..crate::fsops::Meta::default()
            },
        );
        fs.children
            .insert(PathBuf::from("/d/link"), vec![PathBuf::from("/d/link/x")]);
        delete_target(
            &fs,
            Path::new("/d"),
            CleanupKind::DeleteContents,
            Disposition::Permanent,
            false,
        )
        .unwrap();
        let removed = fs.removed.borrow();
        assert!(removed.contains(&PathBuf::from("/d/link")));
        assert!(!removed.contains(&PathBuf::from("/d/link/x")));
    }
}
