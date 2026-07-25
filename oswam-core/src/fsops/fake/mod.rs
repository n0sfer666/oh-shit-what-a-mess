mod ops;

use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct FakeFs {
    pub entries: BTreeMap<PathBuf, Meta>,
    pub children: BTreeMap<PathBuf, Vec<PathBuf>>,
    pub denied: BTreeSet<PathBuf>,
    pub links: BTreeMap<PathBuf, PathBuf>,
    pub home: Option<PathBuf>,
    pub home_alias: Option<PathBuf>,
    pub home_dev: Option<i64>,
    pub trashed: std::cell::RefCell<Vec<PathBuf>>,
    pub removed: std::cell::RefCell<Vec<PathBuf>>,
}

impl FakeFs {
    pub fn rooted_at(&mut self, home: &str) -> &mut Self {
        self.home = Some(PathBuf::from(home));
        self
    }

    pub fn home_aliased_as(&mut self, alias: &str) -> &mut Self {
        self.home_alias = Some(PathBuf::from(alias));
        self
    }

    pub fn home_on_volume(&mut self, dev: i64) -> &mut Self {
        self.home_dev = Some(dev);
        self
    }

    pub fn on_volume(&mut self, path: &str, dev: i64) -> &mut Self {
        let meta = self
            .entries
            .get_mut(Path::new(path))
            .unwrap_or_else(|| panic!("on_volume before the entry exists: {path}"));
        meta.dev = dev;
        self
    }

    pub(super) fn homes(&self) -> Vec<&PathBuf> {
        self.home.iter().chain(self.home_alias.iter()).collect()
    }

    pub(super) fn same_volume(&self, path: &Path) -> bool {
        let Some(home_dev) = self.home_dev else {
            return true;
        };
        self.meta(path).is_ok_and(|m| m.dev == home_dev)
    }

    pub fn deny(&mut self, path: &str) -> &mut Self {
        self.denied.insert(PathBuf::from(path));
        self
    }

    fn still_has_children(&self, path: &Path) -> bool {
        let gone =
            |c: &PathBuf| self.removed.borrow().contains(c) || self.trashed.borrow().contains(c);
        self.children
            .get(path)
            .is_some_and(|kids| kids.iter().any(|c| !gone(c)))
    }

    fn unlinked(&self, meta: &Meta) -> u64 {
        if !meta.is_shared() {
            return 0;
        }
        self.removed
            .borrow()
            .iter()
            .filter_map(|gone| self.entries.get(gone))
            .filter(|m| m.is_shared() && m.ino == meta.ino && m.dev == meta.dev)
            .count() as u64
    }

    fn guard_write(&self, path: &Path) -> io::Result<()> {
        if self.denied.contains(path) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"));
        }
        Ok(())
    }

    pub fn file(&mut self, path: &str, blocks: u64) -> &mut Self {
        self.entries.insert(
            PathBuf::from(path),
            Meta {
                blocks,
                uid: 501,
                ..Meta::default()
            },
        );
        self
    }

    pub fn symlink(&mut self, path: &str, blocks: u64, children: &[&str]) -> &mut Self {
        self.entries.insert(
            PathBuf::from(path),
            Meta {
                is_symlink: true,
                blocks,
                uid: 501,
                ..Meta::default()
            },
        );
        self.children.insert(
            PathBuf::from(path),
            children.iter().map(PathBuf::from).collect(),
        );
        self
    }

    pub fn link(&mut self, path: &str, to: &str) -> &mut Self {
        self.links.insert(PathBuf::from(path), PathBuf::from(to));
        self
    }

    pub fn linked_dir(&mut self, path: &str, to: &str, children: &[&str]) -> &mut Self {
        self.dir(path, children);
        if let Some(meta) = self.entries.get_mut(Path::new(path)) {
            meta.is_symlink = true;
        }
        self.link(path, to)
    }

    pub fn hardlink(&mut self, path: &str, blocks: u64, ino: u64) -> &mut Self {
        self.entries.insert(
            PathBuf::from(path),
            Meta {
                blocks,
                uid: 501,
                nlink: 2,
                ino,
                ..Meta::default()
            },
        );
        self
    }

    pub fn dir(&mut self, path: &str, children: &[&str]) -> &mut Self {
        let p = PathBuf::from(path);
        self.entries.insert(
            p.clone(),
            Meta {
                is_dir: true,
                blocks: 8,
                uid: 501,
                ..Meta::default()
            },
        );
        self.children
            .insert(p, children.iter().map(PathBuf::from).collect());
        self
    }
}
