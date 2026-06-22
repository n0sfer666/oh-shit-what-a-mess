use std::path::{Path, PathBuf};
use std::process::Command;

pub trait ProcessProbe {
    fn holds(&self, path: &Path) -> bool;
}

pub fn path_held(open_paths: &[PathBuf], target: &Path) -> bool {
    open_paths
        .iter()
        .any(|o| o == target || o.starts_with(target))
}

pub struct LsofProbe {
    open_paths: Vec<PathBuf>,
}

impl LsofProbe {
    pub fn collect() -> Self {
        let open_paths = Command::new("lsof")
            .args(["-Fn", "-w"])
            .output()
            .ok()
            .filter(|o| o.status.success() || !o.stdout.is_empty())
            .map(|o| parse_lsof(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_default();
        Self { open_paths }
    }

    pub fn from_paths(open_paths: Vec<PathBuf>) -> Self {
        Self { open_paths }
    }
}

impl ProcessProbe for LsofProbe {
    fn holds(&self, path: &Path) -> bool {
        path_held(&self.open_paths, path)
    }
}

fn parse_lsof(out: &str) -> Vec<PathBuf> {
    out.lines()
        .filter_map(|l| l.strip_prefix('n'))
        .filter(|n| n.starts_with('/'))
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_open_file_under_target() {
        let open = vec![PathBuf::from("/Users/n0sfer/Library/Caches/Arc/x.db")];
        assert!(path_held(
            &open,
            Path::new("/Users/n0sfer/Library/Caches/Arc")
        ));
    }

    #[test]
    fn unrelated_open_files_not_held() {
        let open = vec![PathBuf::from("/Users/n0sfer/Documents/note.txt")];
        assert!(!path_held(
            &open,
            Path::new("/Users/n0sfer/Library/Caches/Arc")
        ));
    }

    #[test]
    fn parses_lsof_name_lines() {
        let out = "p123\nn/Users/x/file\nfcwd\nn/tmp/y\nnpipe\n";
        assert_eq!(
            parse_lsof(out),
            vec![PathBuf::from("/Users/x/file"), PathBuf::from("/tmp/y")]
        );
    }
}
