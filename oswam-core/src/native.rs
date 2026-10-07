use crate::category::NativeSpec;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn command(parts: &[String], privileged: bool) -> Option<Command> {
    let (program, args) = parts.split_first()?;
    let mut cmd = Command::new(program);
    cmd.args(args);
    if !privileged {
        crate::sudo::drop_privileges(&mut cmd);
    }
    Some(cmd)
}

pub fn run(parts: &[String], privileged: bool) -> io::Result<String> {
    let mut cmd = command(parts, privileged)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty native command"))?;
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn run_within(parts: &[String], privileged: bool, limit: Duration) -> io::Result<String> {
    let mut cmd = command(parts, privileged)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty native command"))?;
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("no stdout"))?;
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).map(|_| buf)
    });
    let deadline = Instant::now() + limit;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native command timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let buf = reader
        .join()
        .map_err(|_| io::Error::other("stdout reader panicked"))??;
    if !status.success() {
        return Err(io::Error::other("native command failed"));
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn run_spec(spec: &NativeSpec) -> io::Result<String> {
    let out = run(&spec.clean, spec.privileged);
    crate::docker::clear_estimates();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_command_that_finishes_in_time_returns_its_output() {
        let out = run_within(&argv(&["/bin/echo", "ok"]), true, Duration::from_secs(5));
        assert_eq!(out.expect("echo").trim(), "ok");
    }

    #[test]
    fn a_hung_command_is_killed_at_the_deadline() {
        let started = Instant::now();
        let out = run_within(
            &argv(&["/bin/sleep", "30"]),
            true,
            Duration::from_millis(200),
        );
        assert_eq!(out.expect_err("timeout").kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_failing_command_is_an_error() {
        assert!(run_within(&argv(&["/usr/bin/false"]), true, Duration::from_secs(5)).is_err());
    }
}
