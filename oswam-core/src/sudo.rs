use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    pub uid: u32,
    pub gid: u32,
    pub name: String,
    pub home: Option<PathBuf>,
}

pub fn caller() -> Option<Caller> {
    static CALLER: std::sync::OnceLock<Option<Caller>> = std::sync::OnceLock::new();
    CALLER
        .get_or_init(|| {
            crate::privilege::is_root()
                .then(|| caller_from(|key| std::env::var(key).ok(), passwd_home))
                .flatten()
        })
        .clone()
}

pub fn caller_from<G, H>(get: G, home_of: H) -> Option<Caller>
where
    G: Fn(&str) -> Option<String>,
    H: Fn(&str, u32) -> Option<PathBuf>,
{
    let uid: u32 = get("SUDO_UID")?.parse().ok()?;
    let gid: u32 = get("SUDO_GID")?.parse().ok()?;
    if uid == 0 {
        return None;
    }
    let name = get("SUDO_USER").unwrap_or_default();
    let home = home_of(&name, uid);
    Some(Caller {
        uid,
        gid,
        name,
        home,
    })
}

pub fn user_home() -> Option<PathBuf> {
    resolved_home(caller(), dirs::home_dir())
}

pub fn resolved_home(caller: Option<Caller>, fallback: Option<PathBuf>) -> Option<PathBuf> {
    caller.and_then(|c| c.home).or(fallback)
}

#[cfg(unix)]
fn passwd_home(name: &str, uid: u32) -> Option<PathBuf> {
    use std::ffi::{CStr, CString, OsStr};
    use std::os::unix::ffi::OsStrExt;
    let pw = if name.is_empty() {
        unsafe { libc::getpwuid(uid) }
    } else {
        let c = CString::new(name).ok()?;
        unsafe { libc::getpwnam(c.as_ptr()) }
    };
    if pw.is_null() {
        return None;
    }
    let dir = unsafe { (*pw).pw_dir };
    if dir.is_null() {
        return None;
    }
    let bytes = unsafe { CStr::from_ptr(dir) }.to_bytes();
    if bytes.is_empty() {
        return None;
    }
    Some(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
fn passwd_home(_name: &str, _uid: u32) -> Option<PathBuf> {
    None
}

#[cfg(unix)]
pub fn drop_privileges(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    let Some(caller) = caller() else {
        return;
    };
    cmd.uid(caller.uid).gid(caller.gid);
    if !caller.name.is_empty() {
        cmd.env("USER", &caller.name).env("LOGNAME", &caller.name);
    }
    if let Some(home) = caller.home.as_ref() {
        cmd.env("HOME", home);
    }
}

#[cfg(not(unix))]
pub fn drop_privileges(_cmd: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
        }
    }

    fn passwd(name: &str, _uid: u32) -> Option<PathBuf> {
        (name == "tester").then(|| PathBuf::from("/Users/tester"))
    }

    fn sudoed<'a>(pairs: &'a [(&'a str, &'a str)]) -> Option<Caller> {
        caller_from(env(pairs), passwd)
    }

    #[test]
    fn sudo_reveals_the_user_behind_root() {
        let caller = sudoed(&[
            ("SUDO_UID", "501"),
            ("SUDO_GID", "20"),
            ("SUDO_USER", "tester"),
        ])
        .expect("caller");
        assert_eq!(caller.uid, 501);
        assert_eq!(caller.gid, 20);
        assert_eq!(caller.name, "tester");
        assert_eq!(caller.home, Some(PathBuf::from("/Users/tester")));
    }

    #[test]
    fn a_real_root_login_has_nobody_to_drop_to() {
        assert!(sudoed(&[("SUDO_UID", "0"), ("SUDO_GID", "0")]).is_none());
        assert!(sudoed(&[]).is_none());
        assert!(sudoed(&[("SUDO_UID", "501")]).is_none());
    }

    #[test]
    fn under_sudo_the_home_belongs_to_the_caller_not_to_root() {
        let caller = sudoed(&[
            ("SUDO_UID", "501"),
            ("SUDO_GID", "20"),
            ("SUDO_USER", "tester"),
        ]);
        assert_eq!(
            resolved_home(caller, Some(PathBuf::from("/var/root"))),
            Some(PathBuf::from("/Users/tester"))
        );
    }

    #[test]
    fn an_unresolvable_caller_home_falls_back_to_the_environment() {
        let caller = sudoed(&[
            ("SUDO_UID", "501"),
            ("SUDO_GID", "20"),
            ("SUDO_USER", "ghost"),
        ])
        .expect("caller");
        assert!(caller.home.is_none());
        assert_eq!(
            resolved_home(Some(caller), Some(PathBuf::from("/var/root"))),
            Some(PathBuf::from("/var/root"))
        );
        assert_eq!(
            resolved_home(None, Some(PathBuf::from("/Users/tester"))),
            Some(PathBuf::from("/Users/tester"))
        );
    }
}
