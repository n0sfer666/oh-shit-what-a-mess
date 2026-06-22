use crate::fsops::FsOps;
use std::io;
use std::path::Path;

pub fn physical_size<F: FsOps>(fs: &F, path: &Path) -> io::Result<u64> {
    let meta = fs.meta(path)?;
    let mut total = meta.physical_bytes();
    if meta.is_dir && !meta.is_symlink {
        if let Ok(children) = fs.read_dir(path) {
            for child in children {
                total += physical_size(fs, &child).unwrap_or(0);
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsops::fake::FakeFs;

    #[test]
    fn sums_directory_tree_physically() {
        let mut fs = FakeFs::default();
        fs.dir("/c", &["/c/a", "/c/sub"]);
        fs.file("/c/a", 4);
        fs.dir("/c/sub", &["/c/sub/b"]);
        fs.file("/c/sub/b", 16);
        let bytes = physical_size(&fs, Path::new("/c")).unwrap();
        assert_eq!(bytes, (8 + 4 + 8 + 16) * 512);
    }

    #[test]
    fn inaccessible_child_is_skipped_not_fatal() {
        let mut fs = FakeFs::default();
        fs.dir("/c", &["/c/ok", "/c/denied"]);
        fs.file("/c/ok", 4);
        let bytes = physical_size(&fs, Path::new("/c")).unwrap();
        assert_eq!(bytes, (8 + 4) * 512);
    }

    #[test]
    fn single_file() {
        let mut fs = FakeFs::default();
        fs.file("/f", 3);
        assert_eq!(physical_size(&fs, Path::new("/f")).unwrap(), 1536);
    }
}
