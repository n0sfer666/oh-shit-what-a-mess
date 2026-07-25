use super::FakeFs;
use crate::fsops::{is_inside_trash, FsOps, Meta};
use std::io;
use std::path::{Path, PathBuf};

impl FsOps for FakeFs {
    fn meta(&self, path: &Path) -> io::Result<Meta> {
        let mut meta = self
            .entries
            .get(path)
            .copied()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing"))?;
        meta.nlink = meta.nlink.saturating_sub(self.unlinked(&meta));
        Ok(meta)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.children
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing"))
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.guard_write(path)?;
        self.removed.borrow_mut().push(path.to_path_buf());
        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        self.guard_write(path)?;
        if self.still_has_children(path) {
            return Err(io::Error::other("directory not empty"));
        }
        self.removed.borrow_mut().push(path.to_path_buf());
        Ok(())
    }

    fn move_to_trash(&self, path: &Path) -> io::Result<()> {
        self.guard_write(path)?;
        self.trashed.borrow_mut().push(path.to_path_buf());
        Ok(())
    }

    fn can_trash(&self, path: &Path) -> bool {
        let homes = self.homes();
        if homes.is_empty() {
            return true;
        }
        homes.iter().any(|home| path.starts_with(home))
            && !homes.iter().any(|home| is_inside_trash(path, home))
            && self.same_volume(path)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let mut resolved = path.to_path_buf();
        for _ in 0..8 {
            let hop = self.links.iter().rev().find_map(|(link, dest)| {
                let rest = resolved.strip_prefix(link).ok()?;
                Some(if rest.as_os_str().is_empty() {
                    dest.clone()
                } else {
                    dest.join(rest)
                })
            });
            match hop {
                Some(next) => resolved = next,
                None if self.entries.contains_key(&resolved) => return Ok(resolved),
                None => return Err(io::Error::new(io::ErrorKind::NotFound, "missing")),
            }
        }
        Err(io::Error::new(io::ErrorKind::InvalidInput, "link loop"))
    }
}
