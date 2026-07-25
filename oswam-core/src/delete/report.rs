use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Disposition {
    Trash,
    Permanent,
}

impl Disposition {
    pub fn action_label(&self) -> &'static str {
        match self {
            Disposition::Trash => "trash",
            Disposition::Permanent => "permanent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedItem {
    pub path: PathBuf,
    pub physical_bytes: u64,
    pub disposition: Disposition,
    pub partial: bool,
    pub untouched: bool,
}

#[derive(Debug)]
pub struct DeleteFailure {
    pub path: PathBuf,
    pub error: io::Error,
}

#[derive(Debug, Default)]
pub struct DeleteReport {
    pub done: Vec<DeletedItem>,
    pub failed: Vec<DeleteFailure>,
}

impl DeleteReport {
    pub fn bytes(&self, disposition: Disposition) -> u64 {
        self.done
            .iter()
            .filter(|item| item.disposition == disposition)
            .map(|item| item.physical_bytes)
            .sum()
    }

    pub fn absorb(&mut self, other: DeleteReport) {
        self.done.extend(other.done);
        self.failed.extend(other.failed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(bytes: u64, disposition: Disposition) -> DeletedItem {
        DeletedItem {
            path: PathBuf::from("/Users/tester/x"),
            physical_bytes: bytes,
            disposition,
            partial: false,
            untouched: false,
        }
    }

    #[test]
    fn bytes_are_split_by_actual_disposition() {
        let report = DeleteReport {
            done: vec![
                item(10, Disposition::Trash),
                item(4, Disposition::Permanent),
            ],
            failed: Vec::new(),
        };
        assert_eq!(report.bytes(Disposition::Trash), 10);
        assert_eq!(report.bytes(Disposition::Permanent), 4);
    }

    #[test]
    fn absorb_merges_both_sides() {
        let mut report = DeleteReport {
            done: vec![item(1, Disposition::Trash)],
            failed: Vec::new(),
        };
        report.absorb(DeleteReport {
            done: vec![item(2, Disposition::Trash)],
            failed: vec![DeleteFailure {
                path: PathBuf::from("/Users/tester/y"),
                error: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
            }],
        });
        assert_eq!(report.done.len(), 2);
        assert_eq!(report.failed.len(), 1);
    }
}
