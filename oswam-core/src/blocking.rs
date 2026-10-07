use crate::process::{LsofProbe, ProcessProbe};
use crate::scan::ScanEntry;

pub fn app_in_the_way<'a, P: ProcessProbe>(entry: &'a ScanEntry, probe: &P) -> Option<&'a str> {
    let held = || probe.holds(&entry.path) || entry.companions.iter().any(|c| probe.holds(c));
    entry
        .blocking_app
        .as_deref()
        .filter(|app| probe.running(app) || held())
}

pub struct Recheck(Option<LsofProbe>);

impl Recheck {
    pub fn before_delete(entries: &[ScanEntry]) -> Self {
        Self(
            entries
                .iter()
                .any(|e| e.blocking_app.is_some())
                .then(LsofProbe::collect),
        )
    }

    pub fn blocker<'a>(&self, entry: &'a ScanEntry) -> Option<&'a str> {
        self.0
            .as_ref()
            .and_then(|probe| app_in_the_way(entry, probe))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::CleanupKind;
    use crate::risk::RiskLevel;
    use std::path::PathBuf;

    fn entry(app: Option<&str>) -> ScanEntry {
        ScanEntry {
            display: "rootfs.img".into(),
            path: PathBuf::from("/Users/tester/rootfs.img"),
            kind: CleanupKind::DeletePath,
            risk: RiskLevel::Caution,
            physical_bytes: 1,
            native: None,
            companions: Vec::new(),
            shared_bytes: 0,
            size_unknown: false,
            partial: false,
            needs_root: false,
            permanent_only: false,
            blocking_app: app.map(str::to_string),
            blocked: false,
        }
    }

    #[test]
    fn an_app_reopened_after_the_scan_blocks_the_delete() {
        let probe = LsofProbe::from_paths(vec![]).with_running(&["Claude"]);
        assert_eq!(
            app_in_the_way(&entry(Some("Claude")), &probe),
            Some("Claude")
        );
        assert_eq!(app_in_the_way(&entry(None), &probe), None);
    }

    #[test]
    fn a_closed_app_lets_the_delete_through() {
        let probe = LsofProbe::from_paths(vec![]).with_running(&["Claude Helper"]);
        assert_eq!(app_in_the_way(&entry(Some("Claude")), &probe), None);
    }

    #[test]
    fn a_leftover_process_holding_the_file_blocks_the_delete() {
        let mut e = entry(Some("Claude"));
        let probe = LsofProbe::from_paths(vec![e.path.clone()]);
        assert_eq!(app_in_the_way(&e, &probe), Some("Claude"));
        let manifest = PathBuf::from("/Users/tester/.rootfs.img.origin");
        e.companions = vec![manifest.clone()];
        let probe = LsofProbe::from_paths(vec![manifest]);
        assert_eq!(app_in_the_way(&e, &probe), Some("Claude"));
        assert_eq!(app_in_the_way(&entry(None), &probe), None);
    }

    #[test]
    fn no_probe_runs_when_nothing_waits_for_an_app() {
        assert!(Recheck::before_delete(&[entry(None)]).0.is_none());
    }
}
