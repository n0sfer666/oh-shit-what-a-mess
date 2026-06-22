use std::path::{Path, PathBuf};

pub fn expand_tilde(raw: &str, home: &Path) -> PathBuf {
    if raw == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home.join(rest);
    }
    PathBuf::from(raw)
}

pub fn is_ancestor(ancestor: &Path, descendant: &Path) -> bool {
    ancestor != descendant && descendant.starts_with(ancestor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_home() {
        let home = Path::new("/Users/n0sfer");
        assert_eq!(expand_tilde("~", home), home);
        assert_eq!(
            expand_tilde("~/Library/Caches", home),
            home.join("Library/Caches")
        );
    }

    #[test]
    fn absolute_unchanged() {
        let home = Path::new("/Users/n0sfer");
        assert_eq!(expand_tilde("/System", home), PathBuf::from("/System"));
    }

    #[test]
    fn ancestor_detection() {
        let a = Path::new("/Users/n0sfer/Library/Caches");
        let d = Path::new("/Users/n0sfer/Library/Caches/Yarn");
        assert!(is_ancestor(a, d));
        assert!(!is_ancestor(d, a));
        assert!(!is_ancestor(a, a));
    }
}
