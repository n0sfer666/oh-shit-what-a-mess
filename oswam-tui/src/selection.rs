use crate::app::{App, Panel};
use oswam_core::risk::RiskLevel;
use oswam_core::scan::ScanEntry;
use oswam_core::select::is_deletable;

pub fn is_recommended(entry: &ScanEntry) -> bool {
    is_deletable(entry) && entry.risk == RiskLevel::Safe && !entry.permanent_only
}

pub fn is_bulk_pick(entry: &ScanEntry) -> bool {
    is_deletable(entry) && entry.risk <= RiskLevel::Caution && !entry.permanent_only
}

impl App {
    pub(crate) fn toggle_selection(&mut self) {
        let Some(ci) = self.current_category() else {
            return;
        };
        if self.panel == Panel::Table {
            if let Some(entry) = self.result.categories[ci].entries.get(self.file_cursor) {
                if is_deletable(entry) {
                    let key = (ci, self.file_cursor);
                    if !self.selected.remove(&key) {
                        self.selected.insert(key);
                    }
                }
            }
        } else {
            self.toggle_category(ci);
        }
    }

    pub(crate) fn select_recommended(&mut self) {
        self.selected.clear();
        for (ci, cat) in self.result.categories.iter().enumerate() {
            for (ei, entry) in cat.entries.iter().enumerate() {
                if is_recommended(entry) {
                    self.selected.insert((ci, ei));
                }
            }
        }
    }

    fn toggle_category(&mut self, ci: usize) {
        let bulk: Vec<usize> = self.result.categories[ci]
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| is_bulk_pick(e))
            .map(|(ei, _)| ei)
            .collect();
        if bulk.is_empty() {
            self.selected.retain(|(c, _)| *c != ci);
        } else if bulk.iter().all(|ei| self.selected.contains(&(ci, *ei))) {
            self.selected
                .retain(|(c, ei)| *c != ci || !bulk.contains(ei));
        } else {
            self.selected.extend(bulk.into_iter().map(|ei| (ci, ei)));
        }
    }
}
