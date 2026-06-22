use crate::category::NativeSpec;
use std::io;
use std::process::Command;

pub fn run(command: &[String]) -> io::Result<String> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty native command"))?;
    let out = Command::new(program).args(args).output()?;
    if !out.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn run_spec(spec: &NativeSpec) -> io::Result<String> {
    run(&spec.clean)
}
