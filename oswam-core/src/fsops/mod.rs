#[cfg(test)]
pub(crate) mod fake;
mod real;

pub use real::RealFs;

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
    pub nlink: u64,
    pub ino: u64,
    pub dev: i64,
}

impl Meta {
    pub fn is_shared(&self) -> bool {
        self.nlink > 1
    }

    pub fn physical_bytes(&self) -> u64 {
        self.blocks * BLOCK_SIZE
    }
}

pub fn trash_root(home: &Path) -> PathBuf {
    home.join(".Trash")
}

pub fn is_inside_trash(path: &Path, home: &Path) -> bool {
    path.starts_with(trash_root(home))
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
    fn can_trash(&self, path: &Path) -> bool {
        let _ = path;
        true
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(path.to_path_buf())
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
    fn trash_detection_uses_the_real_trash_root() {
        let home = Path::new("/Users/tester");
        assert!(is_inside_trash(Path::new("/Users/tester/.Trash/old"), home));
        assert!(!is_inside_trash(
            Path::new("/Users/tester/_dev/fixtures/.Trash/x"),
            home
        ));
    }

    #[test]
    fn a_path_on_another_volume_cannot_be_trashed() {
        let mut fs = fake::FakeFs::default();
        fs.rooted_at("/Users/tester").home_on_volume(1);
        fs.dir("/Users/tester/dev", &[]);
        fs.on_volume("/Users/tester/dev", 2);
        fs.dir("/Users/tester/Library", &[]);
        fs.on_volume("/Users/tester/Library", 1);
        assert!(!fs.can_trash(Path::new("/Users/tester/dev")));
        assert!(fs.can_trash(Path::new("/Users/tester/Library")));
    }

    #[test]
    fn an_aliased_home_is_still_recognised_as_home() {
        let mut fs = fake::FakeFs::default();
        fs.rooted_at("/System/Volumes/Data/Users/tester")
            .home_aliased_as("/Users/tester");
        fs.dir("/Users/tester/Library", &[]);
        fs.dir("/Users/tester/.Trash/old", &[]);
        assert!(fs.can_trash(Path::new("/Users/tester/Library")));
        assert!(!fs.can_trash(Path::new("/Users/tester/.Trash/old")));
    }

    #[test]
    fn without_a_known_home_volume_trash_stays_available() {
        let mut fs = fake::FakeFs::default();
        fs.rooted_at("/Users/tester");
        fs.dir("/Users/tester/dev", &[]);
        fs.on_volume("/Users/tester/dev", 2);
        assert!(fs.can_trash(Path::new("/Users/tester/dev")));
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
