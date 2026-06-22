use crate::app::{App, Grouping, Key, Panel, Phase};
use oswam_core::config::Theme;
use oswam_core::delete::Disposition;
use oswam_core::select::is_deletable;

impl App {
    pub fn on_key(&mut self, key: Key) {
        match self.phase {
            Phase::Welcome => self.on_welcome_key(key),
            Phase::Scanning => self.on_scanning_key(key),
            Phase::Results => self.on_results_key(key),
        }
    }

    fn on_welcome_key(&mut self, key: Key) {
        match key {
            Key::Enter | Key::Space | Key::Proceed => self.start_scan_requested = true,
            Key::Quit | Key::Cancel => self.should_quit = true,
            _ => {}
        }
    }

    fn on_scanning_key(&mut self, key: Key) {
        if matches!(key, Key::Quit | Key::Cancel) {
            self.should_quit = true;
        }
    }

    fn on_results_key(&mut self, key: Key) {
        if self.help_visible {
            self.handle_help_key(key);
            return;
        }
        if self.confirm_open {
            self.handle_confirm_key(key);
            return;
        }
        match key {
            Key::Quit => self.should_quit = true,
            Key::Help => self.help_visible = true,
            Key::Theme => self.toggle_theme(),
            Key::Group => self.cycle_grouping(),
            Key::Proceed => self.open_confirm(),
            Key::Tab => self.cycle_panel(),
            Key::Left => self.panel = Panel::Sidebar,
            Key::Right => self.panel = Panel::Table,
            Key::Up => self.move_cursor(-1),
            Key::Down => self.move_cursor(1),
            Key::Top => self.set_cursor(0),
            Key::Bottom => self.set_cursor(usize::MAX),
            Key::Space => self.toggle_selection(),
            Key::Enter | Key::Cancel => {}
        }
    }

    fn open_confirm(&mut self) {
        if self.selected_total_bytes() > 0 {
            self.confirm_open = true;
            self.confirm_choice = 0;
        }
    }

    fn handle_confirm_key(&mut self, key: Key) {
        match key {
            Key::Up | Key::Down => self.confirm_choice ^= 1,
            Key::Enter => {
                self.decision = Some(if self.confirm_choice == 0 {
                    Disposition::Trash
                } else {
                    Disposition::Permanent
                });
            }
            Key::Cancel | Key::Quit => self.confirm_open = false,
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: Key) {
        match key {
            Key::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
            Key::Down => self.help_scroll = self.help_scroll.saturating_add(1),
            Key::Left | Key::Right => {}
            _ => {
                self.help_visible = false;
                self.help_scroll = 0;
            }
        }
    }

    fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
    }

    fn cycle_grouping(&mut self) {
        self.grouping = match self.grouping {
            Grouping::Category => Grouping::Size,
            Grouping::Size => Grouping::Risk,
            Grouping::Risk => Grouping::Category,
        };
        self.category_cursor = 0;
        self.file_cursor = 0;
    }

    fn cycle_panel(&mut self) {
        self.panel = match self.panel {
            Panel::Sidebar => Panel::Description,
            Panel::Description => Panel::Table,
            Panel::Table => Panel::Sidebar,
        };
    }

    fn move_cursor(&mut self, delta: isize) {
        match self.panel {
            Panel::Table => self.file_cursor = step(self.file_cursor, delta, self.entry_count()),
            _ => {
                self.category_cursor =
                    step(self.category_cursor, delta, self.result.categories.len());
                self.file_cursor = 0;
            }
        }
    }

    fn set_cursor(&mut self, pos: usize) {
        match self.panel {
            Panel::Table => self.file_cursor = pos.min(self.entry_count().saturating_sub(1)),
            _ => {
                self.category_cursor = pos.min(self.result.categories.len().saturating_sub(1));
                self.file_cursor = 0;
            }
        }
    }

    fn toggle_selection(&mut self) {
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

fn step(cur: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (cur as isize + delta).clamp(0, len as isize - 1) as usize
}
