use crate::fsops::FsOps;
use crate::paths::{expand_tilde, is_ancestor};
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
    crate::sudo::user_home().map(|h| h.join(".config/oswam/config.toml"))
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
        self.protected_paths.iter().any(|p| {
            let kept = expand_tilde(p, home);
            kept == path || is_ancestor(&kept, path) || is_ancestor(path, &kept)
        })
    }

    pub fn ignore_patterns(&self) -> Vec<glob::Pattern> {
        self.ignore_globs
            .iter()
            .filter_map(|g| glob::Pattern::new(g).ok())
            .collect()
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        matches_any(&self.ignore_patterns(), path)
    }

    pub fn resolved<F: FsOps>(&self, fs: &F, home: &Path) -> Self {
        Self {
            protected_paths: all_forms(&self.protected_paths, fs, home, false),
            ignore_globs: all_forms(&self.ignore_globs, fs, home, true),
            theme: self.theme,
        }
    }
}

fn all_forms<F: FsOps>(raw: &[String], fs: &F, home: &Path, glob: bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in raw {
        for form in derived_forms(fs, item, home, glob) {
            if !out.contains(&form) {
                out.push(form);
            }
        }
    }
    out
}

fn derived_forms<F: FsOps>(fs: &F, item: &str, home: &Path, glob: bool) -> Vec<String> {
    let mut forms = vec![item.to_string()];
    let (literal, rest) = if glob {
        split_wildcard(item)
    } else {
        (item, "")
    };
    let expanded = expand_tilde(literal, home);
    if literal.is_empty() || !expanded.is_absolute() {
        return forms;
    }
    let real = fs.canonicalize(&expanded).ok();
    for base in [Some(expanded), real].into_iter().flatten() {
        forms.push(with_tail(&base, rest, glob));
    }
    forms
}

fn with_tail(base: &Path, tail: &str, glob: bool) -> String {
    let text = base.to_string_lossy();
    let mut out = if glob {
        glob::Pattern::escape(&text)
    } else {
        text.into_owned()
    };
    out.push_str(tail);
    out
}

fn split_wildcard(raw: &str) -> (&str, &str) {
    let Some(magic) = raw.find(['*', '?', '[']) else {
        return (raw, "");
    };
    match raw[..magic].rfind('/') {
        Some(slash) => raw.split_at(slash),
        None => ("", raw),
    }
}

pub fn matches_any(patterns: &[glob::Pattern], path: &Path) -> bool {
    let s = path.to_string_lossy();
    patterns.iter().any(|p| p.matches(&s))
}

#[cfg(test)]
mod tests;
