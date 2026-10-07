use crate::app::{App, Phase, ScanState, Summary};

impl App {
    pub fn update_delete(&mut self, message: String, done: usize, total: usize, freed: u64) {
        self.phase = Phase::Deleting;
        self.delete = ScanState {
            message,
            done,
            total,
            bytes: freed,
        };
    }

    pub fn set_summary(&mut self, summary: Summary) {
        self.phase = Phase::Done;
        self.summary = Some(summary);
    }

    pub fn set_purged(&mut self, purged: Summary) {
        let before = self.summary.unwrap_or_default();
        self.trashed.clear();
        self.set_summary(Summary {
            freed: before.freed + purged.freed,
            trashed: before.trashed.saturating_sub(purged.freed),
            failed: before.failed + purged.failed,
            partial: before.partial + purged.partial,
            kept_in_trash: before.kept_in_trash + purged.untouched,
            ..before
        });
    }

    pub(crate) fn request_purge(&mut self) {
        if !self.trashed.is_empty() {
            self.purge_requested = true;
        }
    }
}
