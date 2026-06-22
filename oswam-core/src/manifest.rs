use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    pub path: PathBuf,
    pub physical_bytes: u64,
    pub action: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn record(&mut self, path: &Path, physical_bytes: u64, action: &str, timestamp: &str) {
        self.entries.push(ManifestEntry {
            path: path.to_path_buf(),
            physical_bytes,
            action: action.to_string(),
            timestamp: timestamp.to_string(),
        });
    }

    pub fn total_bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.physical_bytes).sum()
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_totals() {
        let mut m = Manifest::default();
        m.record(Path::new("/a"), 100, "trash", "2026-06-22T00:00:00Z");
        m.record(Path::new("/b"), 50, "permanent", "2026-06-22T00:00:01Z");
        assert_eq!(m.total_bytes(), 150);
        assert_eq!(m.entries.len(), 2);
    }

    #[test]
    fn serializes_to_json() {
        let mut m = Manifest::default();
        m.record(Path::new("/a"), 1, "trash", "t");
        let json = m.to_json().unwrap();
        assert!(json.contains("\"physical_bytes\": 1"));
        assert!(json.contains("\"action\": \"trash\""));
    }
}
