use crate::app::{App, Grouping, Key, Panel, Phase};
use crate::panels::{description_lines, help_lines};
use oswam_core::config::Theme;
use oswam_core::delete::Disposition;

impl App {
    pub fn on_key(&mut self, key: Key) {
        match self.phase {
            Phase::Welcome => self.on_welcome_key(key),
            Phase::Scanning => self.on_scanning_key(key),
            Phase::Results => self.on_results_key(key),
            Phase::Deleting => {}
            Phase::Done => {
                if matches!(key, Key::Quit | Key::Cancel | Key::Enter) {
                    self.should_quit = true;
                }
            }
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
        if !self.selected.is_empty() {
            self.confirm_open = true;
            self.confirm_choice = 0;
        }
    }

    fn handle_confirm_key(&mut self, key: Key) {
        match key {
            Key::Up | Key::Down => self.confirm_choice ^= 1,
            Key::Enter => {
                self.pending_delete = Some(if self.confirm_choice == 0 {
                    Disposition::Trash
                } else {
                    Disposition::Permanent
                });
                self.confirm_open = false;
            }
            Key::Cancel | Key::Quit => self.confirm_open = false,
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: Key) {
        match key {
            Key::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
            Key::Down => {
                self.help_scroll = self
                    .help_scroll
                    .saturating_add(1)
                    .min(self.help_view.max_scroll(&help_lines()))
            }
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
        self.desc_scroll = 0;
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
            Panel::Description => {
                let shift =
                    i16::try_from(delta).unwrap_or(if delta < 0 { i16::MIN } else { i16::MAX });
                let limit = self.desc_view.max_scroll(&description_lines(self));
                self.desc_scroll = self.desc_scroll.saturating_add_signed(shift).min(limit);
            }
            Panel::Table => {
                self.file_cursor = step(self.file_cursor, delta, self.entry_count());
                self.desc_scroll = 0;
            }
            Panel::Sidebar => {
                self.category_cursor =
                    step(self.category_cursor, delta, self.result.categories.len());
                self.file_cursor = 0;
                self.desc_scroll = 0;
            }
        }
    }

    fn set_cursor(&mut self, pos: usize) {
        match self.panel {
            Panel::Description => self.desc_scroll = 0,
            Panel::Table => {
                self.file_cursor = pos.min(self.entry_count().saturating_sub(1));
                self.desc_scroll = 0;
            }
            Panel::Sidebar => {
                self.category_cursor = pos.min(self.result.categories.len().saturating_sub(1));
                self.file_cursor = 0;
                self.desc_scroll = 0;
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
