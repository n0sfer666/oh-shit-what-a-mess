use crate::paths::expand_tilde;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub protected_paths: Vec<String>,
    pub ignore_globs: Vec<String>,
    pub theme: Option<Theme>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config io: {0}")]
    Io(#[from] std::io::Error),
    #[error("config parse: {0}")]
    Parse(#[from] toml::de::Error),
}

pub fn default_config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".config/oswam/config.toml"))
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn is_protected(&self, path: &Path, home: &Path) -> bool {
        self.protected_paths
            .iter()
            .any(|p| expand_tilde(p, home) == path)
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let s = path.to_string_lossy();
        self.ignore_globs.iter().any(|g| {
            glob::Pattern::new(g)
                .map(|p| p.matches(&s))
                .unwrap_or(false)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn missing_file_is_default() {
        let c = Config::load(Path::new("/nonexistent/oswam.toml")).unwrap();
        assert!(c.protected_paths.is_empty());
        assert!(c.theme.is_none());
    }

    #[test]
    fn parses_toml() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("config.toml");
        std::fs::write(
            &f,
            "protected_paths = [\"~/keep\"]\nignore_globs = [\"*.lock\"]\ntheme = \"dark\"\n",
        )
        .unwrap();
        let c = Config::load(&f).unwrap();
        assert_eq!(c.theme, Some(Theme::Dark));
        assert_eq!(c.protected_paths, vec!["~/keep"]);
    }

    #[test]
    fn protected_matches_expanded() {
        let home = Path::new("/Users/n0sfer");
        let c = Config {
            protected_paths: vec!["~/keep".into()],
            ..Config::default()
        };
        assert!(c.is_protected(&PathBuf::from("/Users/n0sfer/keep"), home));
        assert!(!c.is_protected(&PathBuf::from("/Users/n0sfer/other"), home));
    }

    #[test]
    fn ignored_matches_glob() {
        let c = Config {
            ignore_globs: vec!["**/*.lock".into()],
            ..Config::default()
        };
        assert!(c.is_ignored(Path::new("/a/b/yarn.lock")));
        assert!(!c.is_ignored(Path::new("/a/b/data.txt")));
    }
}
