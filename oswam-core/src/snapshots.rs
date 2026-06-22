use std::process::Command;

pub fn parse_snapshots(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("com.apple.TimeMachine.")
                .map(|s| s.trim_end_matches(".local").to_string())
        })
        .collect()
}

pub fn list_local_snapshots() -> Vec<String> {
    Command::new("tmutil")
        .args(["listlocalsnapshots", "/"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_snapshots(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default()
}

pub fn split_keep_latest(mut snaps: Vec<String>) -> (Vec<String>, Option<String>) {
    snaps.sort();
    let latest = snaps.pop();
    (snaps, latest)
}

pub fn delete_command(date: &str) -> Vec<String> {
    vec![
        "tmutil".to_string(),
        "deletelocalsnapshots".to_string(),
        date.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_snapshot_dates() {
        let out = "Snapshots for volume group containing disk /:\n\
                   com.apple.TimeMachine.2026-06-20-100000.local\n\
                   com.apple.TimeMachine.2026-06-22-120102.local\n";
        assert_eq!(
            parse_snapshots(out),
            vec!["2026-06-20-100000", "2026-06-22-120102"]
        );
    }

    #[test]
    fn keeps_most_recent() {
        let snaps = vec![
            "2026-06-22-120102".to_string(),
            "2026-06-20-100000".to_string(),
            "2026-06-21-110000".to_string(),
        ];
        let (deletable, latest) = split_keep_latest(snaps);
        assert_eq!(latest, Some("2026-06-22-120102".to_string()));
        assert_eq!(deletable, vec!["2026-06-20-100000", "2026-06-21-110000"]);
    }

    #[test]
    fn empty_has_no_latest() {
        let (deletable, latest) = split_keep_latest(vec![]);
        assert!(deletable.is_empty());
        assert!(latest.is_none());
    }

    #[test]
    fn delete_command_shape() {
        assert_eq!(
            delete_command("2026-06-20-100000"),
            vec!["tmutil", "deletelocalsnapshots", "2026-06-20-100000"]
        );
    }
}
