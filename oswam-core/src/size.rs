use crate::fsops::{FsOps, Meta};
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Measure {
    pub bytes: u64,
    pub shared_bytes: u64,
    pub partial: bool,
    pub kept: bool,
}

impl Measure {
    pub fn unknown() -> Self {
        Measure {
            partial: true,
            kept: true,
            ..Measure::default()
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.partial && self.bytes == 0
    }

    fn absorb(&mut self, other: Measure) {
        self.bytes += other.bytes;
        self.shared_bytes += other.shared_bytes;
        self.partial |= other.partial;
        self.kept |= other.kept;
    }
}

pub type Allowed<'a> = &'a dyn Fn(&Path, &Meta) -> bool;

pub fn physical_size<F: FsOps>(fs: &F, path: &Path) -> io::Result<u64> {
    measure(fs, path).map(|m| m.bytes)
}

pub fn measure<F: FsOps>(fs: &F, path: &Path) -> io::Result<Measure> {
    measure_where(fs, path, &|_, _| true)
}

pub fn measure_where<F: FsOps>(fs: &F, path: &Path, allowed: Allowed<'_>) -> io::Result<Measure> {
    let meta = fs.meta(path)?;
    Ok(measure_meta(fs, path, &meta, allowed))
}

pub fn measure_contents<F: FsOps>(
    fs: &F,
    path: &Path,
    allowed: Allowed<'_>,
) -> io::Result<Measure> {
    let meta = fs.meta(path)?;
    if !meta.is_dir || meta.is_symlink {
        return Ok(measure_meta(fs, path, &meta, allowed));
    }
    let children = fs.read_dir(path)?;
    Ok(fold_children(fs, children, allowed))
}

fn measure_meta<F: FsOps>(fs: &F, path: &Path, meta: &Meta, allowed: Allowed<'_>) -> Measure {
    if !meta.is_dir || meta.is_symlink {
        let bytes = own_bytes(meta);
        return Measure {
            bytes,
            shared_bytes: if meta.is_shared() { bytes } else { 0 },
            ..Measure::default()
        };
    }
    let mut total = match fs.read_dir(path) {
        Ok(children) => fold_children(fs, children, allowed),
        Err(_) => Measure::unknown(),
    };
    if !total.kept {
        total.bytes += own_bytes(meta);
    }
    total
}

fn fold_children<F: FsOps>(
    fs: &F,
    children: Vec<std::path::PathBuf>,
    allowed: Allowed<'_>,
) -> Measure {
    let mut total = Measure::default();
    for child in children {
        let Ok(meta) = fs.meta(&child) else {
            total.partial = true;
            total.kept = true;
            continue;
        };
        if !allowed(&child, &meta) {
            total.kept = true;
            continue;
        }
        total.absorb(measure_meta(fs, &child, &meta, allowed));
    }
    total
}

pub fn own_bytes(meta: &Meta) -> u64 {
    if meta.is_dir || !meta.is_shared() {
        return meta.physical_bytes();
    }
    meta.physical_bytes() / meta.nlink.max(1)
}

#[cfg(test)]
mod tests;
