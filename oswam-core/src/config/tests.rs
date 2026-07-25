use super::*;
use crate::fsops::fake::FakeFs;
use std::path::PathBuf;

const HOME: &str = "/Users/tester";

fn aliased(link: &str, real: &str) -> FakeFs {
    let mut fs = FakeFs::default();
    fs.dir(real, &[&format!("{real}/p")]);
    fs.dir(&format!("{real}/p"), &[]);
    fs.link(link, real);
    fs
}

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
    let home = Path::new(HOME);
    let c = Config {
        protected_paths: vec!["~/keep".into()],
        ..Config::default()
    };
    assert!(c.is_protected(&PathBuf::from("/Users/tester/keep"), home));
    assert!(!c.is_protected(&PathBuf::from("/Users/tester/other"), home));
}

#[test]
fn protection_covers_the_whole_subtree() {
    let home = Path::new(HOME);
    let c = Config {
        protected_paths: vec!["~/keep".into()],
        ..Config::default()
    };
    assert!(c.is_protected(&PathBuf::from("/Users/tester/keep/deep/file"), home));
}

#[test]
fn a_parent_of_a_protected_path_is_protected_too() {
    let home = Path::new(HOME);
    let c = Config {
        protected_paths: vec!["~/cache/keep".into()],
        ..Config::default()
    };
    assert!(c.is_protected(&PathBuf::from("/Users/tester/cache"), home));
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

#[test]
fn an_aliased_protected_path_also_protects_the_real_one() {
    let fs = aliased("/Users/tester/code", "/Users/tester/work");
    let c = Config {
        protected_paths: vec!["~/code/p".into()],
        ..Config::default()
    }
    .resolved(&fs, Path::new(HOME));
    assert!(c.is_protected(
        &PathBuf::from("/Users/tester/work/p/node_modules"),
        Path::new(HOME)
    ));
    assert!(c.is_protected(
        &PathBuf::from("/Users/tester/code/p/node_modules"),
        Path::new(HOME)
    ));
}

#[test]
fn an_aliased_ignore_glob_also_matches_the_real_path() {
    let fs = aliased("/Users/tester/code", "/Users/tester/work");
    let c = Config {
        ignore_globs: vec!["~/code/**/tmp".into()],
        ..Config::default()
    }
    .resolved(&fs, Path::new(HOME));
    assert!(c.is_ignored(Path::new("/Users/tester/work/p/tmp")));
}

#[test]
fn a_resolved_glob_form_keeps_brackets_literal() {
    let mut fs = FakeFs::default();
    fs.dir("/Volumes/My [Backup]/stuff", &[]);
    fs.link("/Users/tester/ext", "/Volumes/My [Backup]/stuff");
    let c = Config {
        ignore_globs: vec!["~/ext".into()],
        ..Config::default()
    }
    .resolved(&fs, Path::new(HOME));
    assert!(c.is_ignored(Path::new("/Volumes/My [Backup]/stuff")));
}

#[test]
fn a_tilde_glob_matches_the_expanded_path_even_without_resolution() {
    let c = Config {
        ignore_globs: vec!["~/.npm".into()],
        ..Config::default()
    }
    .resolved(&FakeFs::default(), Path::new(HOME));
    assert!(c.is_ignored(Path::new("/Users/tester/.npm")));
}

#[test]
fn a_real_path_that_is_not_an_alias_is_not_duplicated() {
    let mut fs = FakeFs::default();
    fs.dir("/Users/tester/keep", &[]);
    let c = Config {
        protected_paths: vec!["~/keep".into()],
        ..Config::default()
    }
    .resolved(&fs, Path::new(HOME));
    assert_eq!(c.protected_paths, vec!["~/keep", "/Users/tester/keep"]);
}

#[test]
fn a_prefix_that_does_not_exist_keeps_only_the_written_forms() {
    let c = Config {
        protected_paths: vec!["~/gone".into()],
        ..Config::default()
    }
    .resolved(&FakeFs::default(), Path::new(HOME));
    assert_eq!(c.protected_paths, vec!["~/gone", "/Users/tester/gone"]);
    assert!(c.is_protected(&PathBuf::from("/Users/tester/gone"), Path::new(HOME)));
}

#[test]
fn a_relative_entry_is_never_resolved_against_the_current_directory() {
    let c = Config {
        protected_paths: vec!["cache".into()],
        ignore_globs: vec!["build/*".into()],
        ..Config::default()
    }
    .resolved(&FakeFs::default(), Path::new(HOME));
    assert_eq!(c.protected_paths, vec!["cache"]);
    assert_eq!(c.ignore_globs, vec!["build/*"]);
}

#[test]
fn a_glob_without_a_literal_prefix_survives_resolution() {
    let c = Config {
        ignore_globs: vec!["**/*.lock".into()],
        ..Config::default()
    }
    .resolved(&FakeFs::default(), Path::new(HOME));
    assert_eq!(c.ignore_globs, vec!["**/*.lock".to_string()]);
    assert!(c.is_ignored(Path::new("/a/b/yarn.lock")));
}
