use super::{is_inside_trash, trash_root, FsOps, Meta};
use std::io;
use std::path::{Path, PathBuf};

pub struct RealFs {
    home: PathBuf,
    alias: PathBuf,
    trash: PathBuf,
    home_dev: Option<i64>,
}

impl RealFs {
    pub fn new() -> io::Result<Self> {
        let alias = crate::sudo::user_home()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home dir"))?;
        let home = std::fs::canonicalize(&alias).unwrap_or_else(|_| alias.clone());
        let home_dev = std::fs::metadata(&home)
            .ok()
            .map(|m| i64::from(std::os::unix::fs::MetadataExt::dev(&m) as i32));
        Ok(Self {
            trash: trash_root(&home),
            alias,
            home,
            home_dev,
        })
    }

    fn inside_home(&self, path: &Path) -> bool {
        path.starts_with(&self.home) || path.starts_with(&self.alias)
    }

    fn inside_trash(&self, path: &Path) -> bool {
        is_inside_trash(path, &self.home) || is_inside_trash(path, &self.alias)
    }

    #[cfg(unix)]
    fn same_volume(&self, path: &Path) -> bool {
        let Some(home_dev) = self.home_dev else {
            return true;
        };
        self.meta(path).is_ok_and(|m| m.dev == home_dev)
    }

    #[cfg(unix)]
    fn ensure_trash(&self) -> io::Result<()> {
        match std::fs::symlink_metadata(&self.trash) {
            Ok(meta) if meta.is_dir() => return Ok(()),
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "trash is not a directory",
                ))
            }
            Err(_) => {}
        }
        std::os::unix::fs::DirBuilderExt::mode(&mut std::fs::DirBuilder::new(), 0o700)
            .create(&self.trash)?;
        let Some(caller) = crate::sudo::caller() else {
            return Ok(());
        };
        match own_directory(&self.trash, caller.uid, caller.gid) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_dir(&self.trash);
                Err(e)
            }
        }
    }
}

#[cfg(unix)]
fn own_directory(dir: &Path, uid: u32, gid: u32) -> io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c = CString::new(dir.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path has nul"))?;
    let fd = unsafe {
        libc::open(
            c.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let rc = unsafe { libc::fchown(fd, uid, gid) };
    unsafe { libc::close(fd) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
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
            nlink: u64::from(st.st_nlink),
            ino: st.st_ino,
            dev: i64::from(st.st_dev),
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
        if !self.can_trash(path) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot be trashed",
            ));
        }
        self.ensure_trash()?;
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?;
        let mut dest = self.trash.join(name);
        let mut n = 1;
        while std::fs::symlink_metadata(&dest).is_ok() {
            let mut alt = name.to_os_string();
            alt.push(format!(" {n}"));
            dest = self.trash.join(alt);
            n += 1;
        }
        std::fs::rename(path, dest)
    }

    fn can_trash(&self, path: &Path) -> bool {
        self.inside_home(path) && !self.inside_trash(path) && self.same_volume(path)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }
}
