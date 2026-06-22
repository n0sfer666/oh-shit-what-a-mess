use std::io;
use std::path::{Path, PathBuf};

pub const SF_DATALESS: u32 = 0x4000_0000;
pub const SF_RESTRICTED: u32 = 0x0008_0000;
pub const BLOCK_SIZE: u64 = 512;

#[derive(Debug, Clone, Copy, Default)]
pub struct Meta {
    pub is_dir: bool,
    pub is_symlink: bool,
    pub blocks: u64,
    pub uid: u32,
    pub flags: u32,
}

impl Meta {
    pub fn physical_bytes(&self) -> u64 {
        self.blocks * BLOCK_SIZE
    }
}

pub fn is_root_owned(uid: u32) -> bool {
    uid == 0
}

pub fn is_dataless(flags: u32) -> bool {
    flags & SF_DATALESS != 0
}

pub fn is_sip_protected(flags: u32) -> bool {
    flags & SF_RESTRICTED != 0
}

pub trait FsOps {
    fn meta(&self, path: &Path) -> io::Result<Meta>;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn remove_dir(&self, path: &Path) -> io::Result<()>;
    fn move_to_trash(&self, path: &Path) -> io::Result<()>;
}

pub struct RealFs {
    trash: PathBuf,
}

impl RealFs {
    pub fn new() -> io::Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home dir"))?;
        Ok(Self {
            trash: home.join(".Trash"),
        })
    }
}

#[cfg(unix)]
impl FsOps for RealFs {
    fn meta(&self, path: &Path) -> io::Result<Meta> {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c = CString::new(path.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path has nul"))?;
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::lstat(c.as_ptr(), &mut st) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mode = u32::from(st.st_mode) & u32::from(libc::S_IFMT);
        Ok(Meta {
            is_dir: mode == u32::from(libc::S_IFDIR),
            is_symlink: mode == u32::from(libc::S_IFLNK),
            blocks: st.st_blocks.max(0) as u64,
            uid: st.st_uid,
            flags: st.st_flags,
        })
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path)? {
            out.push(entry?.path());
        }
        Ok(out)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir(path)
    }

    fn move_to_trash(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(&self.trash)?;
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?;
        let mut dest = self.trash.join(name);
        let mut n = 1;
        while dest.exists() {
            let mut alt = name.to_os_string();
            alt.push(format!(" {n}"));
            dest = self.trash.join(alt);
            n += 1;
        }
        std::fs::rename(path, dest)
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    pub struct FakeFs {
        pub entries: BTreeMap<PathBuf, Meta>,
        pub children: BTreeMap<PathBuf, Vec<PathBuf>>,
        pub trashed: std::cell::RefCell<Vec<PathBuf>>,
        pub removed: std::cell::RefCell<Vec<PathBuf>>,
    }

    impl FakeFs {
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

    impl FsOps for FakeFs {
        fn meta(&self, path: &Path) -> io::Result<Meta> {
            self.entries
                .get(path)
                .copied()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing"))
        }

        fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
            Ok(self.children.get(path).cloned().unwrap_or_default())
        }

        fn remove_file(&self, path: &Path) -> io::Result<()> {
            self.removed.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn remove_dir(&self, path: &Path) -> io::Result<()> {
            self.removed.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn move_to_trash(&self, path: &Path) -> io::Result<()> {
            self.trashed.borrow_mut().push(path.to_path_buf());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_helpers() {
        assert!(is_root_owned(0));
        assert!(!is_root_owned(501));
        assert!(is_dataless(SF_DATALESS));
        assert!(!is_dataless(0));
        assert!(is_sip_protected(SF_RESTRICTED));
        assert!(is_sip_protected(SF_RESTRICTED | SF_DATALESS));
    }

    #[test]
    fn physical_bytes_from_blocks() {
        let m = Meta {
            blocks: 10,
            ..Meta::default()
        };
        assert_eq!(m.physical_bytes(), 5120);
    }
}
