use crate::app::{max_risk, App, Grouping};
use oswam_core::scan::ScanEntry;

impl App {
    pub fn ordered_categories(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.result.categories.len()).collect();
        match self.grouping {
            Grouping::Category => {}
            Grouping::Size => {
                idx.sort_by_key(|i| std::cmp::Reverse(self.result.categories[*i].total_bytes))
            }
            Grouping::Risk => idx.sort_by_key(|i| std::cmp::Reverse(max_risk(&self.result, *i))),
        }
        idx
    }

    pub fn current_category(&self) -> Option<usize> {
        self.ordered_categories().get(self.category_cursor).copied()
    }

    pub fn entry_count(&self) -> usize {
        self.current_category()
            .map(|ci| self.result.categories[ci].entries.len())
            .unwrap_or(0)
    }

    pub fn selected_total_bytes(&self) -> u64 {
        self.selected
            .iter()
            .filter_map(|(ci, ei)| self.result.categories.get(*ci)?.entries.get(*ei))
            .map(|e| e.physical_bytes)
            .sum()
    }

    pub fn is_selected(&self, ci: usize, ei: usize) -> bool {
        self.selected.contains(&(ci, ei))
    }

    pub fn selected_entries(&self) -> Vec<ScanEntry> {
        let mut out = Vec::new();
        for (ci, cat) in self.result.categories.iter().enumerate() {
            for (ei, entry) in cat.entries.iter().enumerate() {
                if self.is_selected(ci, ei) {
                    out.push(entry.clone());
                }
            }
        }
        out
    }
}
