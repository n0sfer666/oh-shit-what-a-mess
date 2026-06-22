use oswam_core::config::Theme;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::ScanResult;
use oswam_core::select::is_deletable;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Sidebar,
    Description,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    Category,
    Size,
    Risk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
    Space,
    Tab,
    Help,
    Theme,
    Group,
    Quit,
    Proceed,
}

pub struct App {
    pub result: ScanResult,
    pub theme: Theme,
    pub panel: Panel,
    pub grouping: Grouping,
    pub help_visible: bool,
    pub help_scroll: u16,
    pub category_cursor: usize,
    pub file_cursor: usize,
    pub selected: HashSet<(usize, usize)>,
    pub should_quit: bool,
    pub proceed_requested: bool,
}

impl App {
    pub fn new(result: ScanResult, theme: Theme, first_run: bool) -> Self {
        let mut selected = HashSet::new();
        for (ci, cat) in result.categories.iter().enumerate() {
            for (ei, entry) in cat.entries.iter().enumerate() {
                if is_deletable(entry) && entry.risk == RiskLevel::Safe {
                    selected.insert((ci, ei));
                }
            }
        }
        Self {
            result,
            theme,
            panel: Panel::Sidebar,
            grouping: Grouping::Category,
            help_visible: first_run,
            help_scroll: 0,
            category_cursor: 0,
            file_cursor: 0,
            selected,
            should_quit: false,
            proceed_requested: false,
        }
    }

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
}

pub(crate) fn max_risk(result: &ScanResult, ci: usize) -> RiskLevel {
    result.categories[ci]
        .entries
        .iter()
        .map(|e| e.risk)
        .max()
        .unwrap_or(RiskLevel::Safe)
}
