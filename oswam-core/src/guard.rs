use crate::config::{matches_any, Config};
use crate::facts::facts_from_meta;
use crate::fsops::{FsOps, Meta};
use crate::risk::{classify, RiskLevel};
use std::path::Path;

pub struct Guard<'a, F: FsOps> {
    pub fs: &'a F,
    pub config: &'a Config,
    pub home: &'a Path,
    pub allow_root_owned: bool,
    ignore: Vec<glob::Pattern>,
}

impl<'a, F: FsOps> Guard<'a, F> {
    pub fn new(fs: &'a F, config: &'a Config, home: &'a Path) -> Self {
        Self {
            fs,
            config,
            home,
            allow_root_owned: false,
            ignore: config.ignore_patterns(),
        }
    }

    pub fn allowing_root_owned(self, allow: bool) -> Self {
        Self {
            allow_root_owned: allow,
            ..self
        }
    }

    pub fn allows(&self, path: &Path) -> bool {
        let Ok(meta) = self.fs.meta(path) else {
            return false;
        };
        self.allows_meta(path, &meta)
    }

    pub fn allows_meta(&self, path: &Path, meta: &Meta) -> bool {
        if matches_any(&self.ignore, path) || self.config.is_protected(path, self.home) {
            return false;
        }
        let mut facts = facts_from_meta(path, meta, false, false);
        if self.allow_root_owned {
            facts.is_root_owned = false;
        }
        classify(&facts, RiskLevel::Safe) != RiskLevel::Never
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsops::fake::FakeFs;
    use crate::fsops::Meta;
    use std::path::PathBuf;

    fn user_meta() -> Meta {
        Meta {
            uid: 501,
            ..Meta::default()
        }
    }

    fn fs_with(path: &str, meta: Meta) -> FakeFs {
        let mut fs = FakeFs::default();
        fs.dir(path, &[]);
        fs.entries.insert(PathBuf::from(path), meta);
        fs
    }

    fn guard<'a>(fs: &'a FakeFs, config: &'a Config, elevated: bool) -> Guard<'a, FakeFs> {
        Guard::new(fs, config, Path::new("/Users/tester")).allowing_root_owned(elevated)
    }

    #[test]
    fn plain_child_is_allowed() {
        let fs = fs_with("/Users/tester/.cache/x", user_meta());
        let config = Config::default();
        assert!(guard(&fs, &config, false).allows(Path::new("/Users/tester/.cache/x")));
    }

    #[test]
    fn protected_child_is_refused() {
        let fs = fs_with("/Users/tester/.cache/keep", user_meta());
        let config = Config {
            protected_paths: vec!["~/.cache/keep".into()],
            ..Config::default()
        };
        assert!(!guard(&fs, &config, false).allows(Path::new("/Users/tester/.cache/keep")));
    }

    #[test]
    fn ignored_child_is_refused() {
        let fs = fs_with("/Users/tester/.cache/a.lock", user_meta());
        let config = Config {
            ignore_globs: vec!["**/*.lock".into()],
            ..Config::default()
        };
        assert!(!guard(&fs, &config, false).allows(Path::new("/Users/tester/.cache/a.lock")));
    }

    #[test]
    fn never_signals_on_child_are_refused() {
        let sip = Meta {
            flags: crate::fsops::SF_RESTRICTED,
            ..user_meta()
        };
        let fs = fs_with("/Users/tester/.cache/sip", sip);
        let config = Config::default();
        assert!(!guard(&fs, &config, true).allows(Path::new("/Users/tester/.cache/sip")));
    }

    #[test]
    fn root_owned_child_needs_elevation() {
        let fs = fs_with("/Library/Caches/x", Meta::default());
        let config = Config::default();
        assert!(!guard(&fs, &config, false).allows(Path::new("/Library/Caches/x")));
        assert!(guard(&fs, &config, true).allows(Path::new("/Library/Caches/x")));
    }

    #[test]
    fn missing_child_is_refused() {
        let fs = FakeFs::default();
        let config = Config::default();
        assert!(!guard(&fs, &config, true).allows(Path::new("/nope")));
    }
}
