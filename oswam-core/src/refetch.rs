use std::path::Path;

pub const REFETCHED_CACHE_NAMES: [&str; 43] = [
    "homebrew",
    "yarn",
    "pnpm",
    "bun",
    "npm",
    "pip",
    "cocoapods",
    "go-build",
    "deno",
    "uv",
    "ms-playwright",
    "node-gyp",
    "electron",
    "puppeteer",
    "typescript",
    "org.swift.swiftpm",
    "gradle",
    "maven",
    "composer",
    "bazel",
    "flutter",
    "dart",
    "selenium",
    "nuget",
    "cypress",
    "coursier",
    "sccache",
    "ccache",
    "pipenv",
    "virtualenv",
    "poetry",
    "jetbrains",
    "rustup",
    "cargo",
    "zig",
    "huggingface",
    "pre-commit",
    "mise",
    "asdf",
    "sbt",
    "conan",
    "vcpkg",
    "carthage",
];

pub fn costs_a_download(path: &Path) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .is_some_and(|name| REFETCHED_CACHE_NAMES.contains(&name.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_cache_is_recognised_case_insensitively() {
        assert!(costs_a_download(Path::new(
            "/Users/tester/Library/Caches/Homebrew"
        )));
        assert!(costs_a_download(Path::new(
            "/Users/tester/Library/Caches/go-build"
        )));
        assert!(costs_a_download(Path::new("/Users/tester/.cache/pip")));
    }

    #[test]
    fn a_multi_gigabyte_tool_cache_is_never_auto_selected() {
        for name in [
            "Cypress",
            "Coursier",
            "sccache",
            "pipenv",
            "virtualenv",
            "JetBrains",
            "rustup",
            "zig",
        ] {
            assert!(
                costs_a_download(&Path::new("/Users/tester/Library/Caches").join(name)),
                "{name}"
            );
        }
    }

    #[test]
    fn an_app_cache_is_not_mistaken_for_a_download() {
        assert!(!costs_a_download(Path::new(
            "/Users/tester/Library/Caches/com.apple.Safari"
        )));
        assert!(!costs_a_download(Path::new("/Users/tester/Library/Caches")));
    }
}
