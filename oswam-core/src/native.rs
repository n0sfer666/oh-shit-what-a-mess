use crate::category::NativeSpec;
use std::io;
use std::process::Command;

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

pub fn run_spec(spec: &NativeSpec) -> io::Result<String> {
    let out = run(&spec.clean, spec.privileged);
    crate::docker::clear_estimates();
    out
}
