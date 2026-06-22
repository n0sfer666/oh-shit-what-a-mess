use crate::category::NativeSpec;
use std::io;
use std::process::Command;

pub fn parse_human_size(raw: &str) -> Option<u64> {
    let s = raw.trim();
    let num_end = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, unit) = s.split_at(num_end);
    let value: f64 = num.trim().parse().ok()?;
    let mult = match unit.trim().to_ascii_uppercase().as_str() {
        "B" | "" => 1.0,
        "KB" | "K" => 1e3,
        "MB" | "M" => 1e6,
        "GB" | "G" => 1e9,
        "TB" | "T" => 1e12,
        "KIB" => 1024.0,
        "MIB" => 1024.0 * 1024.0,
        "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TIB" => 1024.0_f64.powi(4),
        _ => return None,
    };
    Some((value * mult) as u64)
}

pub fn parse_reclaimable(output: &str) -> u64 {
    output
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter_map(parse_human_size)
        .sum()
}

pub fn estimate(spec: &NativeSpec) -> Option<u64> {
    let mut cmd = build_command(&spec.estimate)?;
    cmd.args(["--format", "{{.Reclaimable}}"]);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(parse_reclaimable(&String::from_utf8_lossy(&out.stdout)))
}

pub fn run_clean(spec: &NativeSpec) -> io::Result<String> {
    let mut cmd = build_command(&spec.clean)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty native command"))?;
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn build_command(parts: &[String]) -> Option<Command> {
    let (program, args) = parts.split_first()?;
    let mut cmd = Command::new(program);
    cmd.args(args);
    Some(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        assert_eq!(parse_human_size("0B"), Some(0));
        assert_eq!(parse_human_size("1.5GB"), Some(1_500_000_000));
        assert_eq!(parse_human_size("100MB"), Some(100_000_000));
        assert_eq!(parse_human_size("2KiB"), Some(2048));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_human_size("abc"), None);
        assert_eq!(parse_human_size("5XB"), None);
    }

    #[test]
    fn sums_reclaimable_column() {
        let out = "1.5GB (50%)\n200MB (10%)\n0B (0%)\n";
        assert_eq!(parse_reclaimable(out), 1_700_000_000);
    }
}
