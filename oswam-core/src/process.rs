use std::path::{Path, PathBuf};
use std::process::Command;

pub trait ProcessProbe {
    fn holds(&self, path: &Path) -> bool;
    fn running(&self, app: &str) -> bool;
}

pub fn path_held(open_paths: &[PathBuf], target: &Path) -> bool {
    open_paths
        .iter()
        .any(|o| o == target || o.starts_with(target))
}

pub struct LsofProbe {
    open_paths: Vec<PathBuf>,
    commands: Option<Vec<String>>,
}

impl LsofProbe {
    pub fn collect() -> Self {
        let out = Command::new("lsof")
            .args(["-Fcn", "-w", "+c", "0"])
            .output()
            .ok()
            .filter(|o| o.status.success() || !o.stdout.is_empty())
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
        Self::from_output(out.as_deref())
    }

    fn from_output(out: Option<&str>) -> Self {
        Self {
            open_paths: out.map(parse_lsof).unwrap_or_default(),
            commands: out.map(parse_commands),
        }
    }

    pub fn from_paths(open_paths: Vec<PathBuf>) -> Self {
        Self {
            open_paths,
            commands: Some(Vec::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_running(mut self, apps: &[&str]) -> Self {
        self.commands = Some(apps.iter().map(|a| a.to_string()).collect());
        self
    }
}

impl ProcessProbe for LsofProbe {
    fn holds(&self, path: &Path) -> bool {
        path_held(&self.open_paths, path)
    }

    fn running(&self, app: &str) -> bool {
        match &self.commands {
            Some(commands) => commands.iter().any(|c| c == app),
            None => true,
        }
    }
}

fn parse_lsof(out: &str) -> Vec<PathBuf> {
    out.lines()
        .filter_map(|l| l.strip_prefix('n'))
        .filter(|n| n.starts_with('/'))
        .map(PathBuf::from)
        .collect()
}

fn parse_commands(out: &str) -> Vec<String> {
    out.lines()
        .filter_map(|l| l.strip_prefix('c'))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_open_file_under_target() {
        let open = vec![PathBuf::from("/Users/tester/Library/Caches/Arc/x.db")];
        assert!(path_held(
            &open,
            Path::new("/Users/tester/Library/Caches/Arc")
        ));
    }

    #[test]
    fn unrelated_open_files_not_held() {
        let open = vec![PathBuf::from("/Users/tester/Documents/note.txt")];
        assert!(!path_held(
            &open,
            Path::new("/Users/tester/Library/Caches/Arc")
        ));
    }

    #[test]
    fn parses_lsof_name_lines() {
        let out = "p123\nn/Users/tester/file\nfcwd\nn/tmp/y\nnpipe\n";
        assert_eq!(
            parse_lsof(out),
            vec![PathBuf::from("/Users/tester/file"), PathBuf::from("/tmp/y")]
        );
    }

    #[test]
    fn parses_lsof_command_lines() {
        let out = "p1\ncClaude\nfcwd\nn/\np2\ncClaude Helper\nftxt\nn/bin/x\n";
        assert_eq!(parse_commands(out), vec!["Claude", "Claude Helper"]);
    }

    #[test]
    fn running_matches_the_exact_process_name() {
        let probe = LsofProbe::from_paths(vec![]).with_running(&["Claude Helper", "claude"]);
        assert!(!probe.running("Claude"));
        assert!(probe.with_running(&["Claude"]).running("Claude"));
    }

    #[test]
    fn an_app_counts_as_running_when_lsof_gave_nothing() {
        assert!(LsofProbe::from_output(None).running("Claude"));
        assert!(!LsofProbe::from_output(Some("p1\ncFinder\n")).running("Claude"));
    }
}
