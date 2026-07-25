use crate::app::{App, Panel};
use oswam_core::select::is_deletable;

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

    fn toggle_category(&mut self, ci: usize) {
        let deletable: Vec<usize> = self.result.categories[ci]
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| is_deletable(e))
            .map(|(ei, _)| ei)
            .collect();
        let all_selected = deletable
            .iter()
            .all(|ei| self.selected.contains(&(ci, *ei)));
        for ei in deletable {
            if all_selected {
                self.selected.remove(&(ci, ei));
            } else {
                self.selected.insert((ci, ei));
            }
        }
    }
}
