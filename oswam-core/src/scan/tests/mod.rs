mod basics;
mod discovered;
mod grouped;
mod native;
mod risk;

use super::*;
use crate::category::Target;
use crate::fsops::fake::FakeFs;
use crate::process::LsofProbe;

const HOME: &str = "/Users/tester";

pub(super) struct Env {
    fs: FakeFs,
    probe: LsofProbe,
    config: Config,
    elevated: bool,
}

impl Env {
    pub(super) fn new(fs: FakeFs) -> Self {
        Self {
            fs,
            probe: LsofProbe::from_paths(vec![]),
            config: Config::default(),
            elevated: false,
        }
    }

    pub(super) fn holding(fs: FakeFs, held: &str) -> Self {
        Self {
            probe: LsofProbe::from_paths(vec![PathBuf::from(held)]),
            ..Self::new(fs)
        }
    }

    pub(super) fn elevated(fs: FakeFs) -> Self {
        Self {
            elevated: true,
            ..Self::new(fs)
        }
    }

    pub(super) fn protecting(fs: FakeFs, kept: &str) -> Self {
        let config = Config {
            protected_paths: vec![kept.to_string()],
            ..Config::default()
        }
        .resolved(&fs, Path::new(HOME));
        Self {
            config,
            ..Self::new(fs)
        }
    }

    pub(super) fn scan(&self, cats: &[Category]) -> ScanResult {
        scan(&self.ctx(), cats, |_| None)
    }

    pub(super) fn ctx(&self) -> ScanCtx<'_, FakeFs, LsofProbe> {
        ScanCtx {
            fs: &self.fs,
            probe: &self.probe,
            config: &self.config,
            home: Path::new(HOME),
            elevated: self.elevated,
        }
    }
}

pub(super) fn one(id: &'static str, target: Target) -> Vec<Category> {
    vec![Category {
        id,
        name: "N",
        glyph: "g",
        targets: vec![target],
    }]
}
