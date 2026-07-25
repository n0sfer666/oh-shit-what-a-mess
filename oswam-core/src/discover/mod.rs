use crate::fsops::FsOps;
use std::path::{Path, PathBuf};

pub fn find_named_dirs<F: FsOps>(
    fs: &F,
    root: &Path,
    names: &[&str],
    max_depth: usize,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(fs, root, names, max_depth, &mut out);
    out
}

const BUILD_MARKERS: [&str; 6] = [
    "CMakeCache.txt",
    "CMakeFiles",
    "build.ninja",
    ".ninja_deps",
    "CACHEDIR.TAG",
    ".rustc_info.json",
];

const AMBIGUOUS_NAMES: [&str; 1] = ["target"];

const PROJECT_MANIFESTS: [&str; 5] = [
    "Cargo.toml",
    "pom.xml",
    "build.sbt",
    "build.gradle",
    "build.gradle.kts",
];

#[derive(PartialEq, Eq)]
enum Match {
    Exact,
    Prefix,
    None,
}

fn name_matches(name: &str, patterns: &[&str]) -> Match {
    for p in patterns {
        match p.strip_suffix('*') {
            Some(prefix) if !prefix.is_empty() && name.starts_with(prefix) => return Match::Prefix,
            Some(_) => continue,
            None if name == *p => return Match::Exact,
            None => continue,
        }
    }
    Match::None
}

fn has_child<F: FsOps>(fs: &F, dir: &Path, names: &[&str]) -> bool {
    let Ok(children) = fs.read_dir(dir) else {
        return false;
    };
    children.iter().any(|c| {
        c.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| names.contains(&n))
    })
}

fn looks_generated<F: FsOps>(fs: &F, dir: &Path) -> bool {
    has_child(fs, dir, &BUILD_MARKERS)
}

fn walk<F: FsOps>(fs: &F, dir: &Path, names: &[&str], depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(children) = fs.read_dir(dir) else {
        return;
    };
    for child in children {
        let Ok(meta) = fs.meta(&child) else {
            continue;
        };
        if !meta.is_dir || meta.is_symlink {
            continue;
        }
        let name = child
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let accepted = match name_matches(name, names) {
            Match::Exact if AMBIGUOUS_NAMES.contains(&name) => {
                looks_generated(fs, &child) || has_child(fs, dir, &PROJECT_MANIFESTS)
            }
            Match::Exact => true,
            Match::Prefix => looks_generated(fs, &child),
            Match::None => false,
        };
        if accepted {
            out.push(child);
            continue;
        }
        if depth > 0 {
            walk(fs, &child, names, depth - 1, out);
        }
    }
}

#[cfg(test)]
mod tests;
