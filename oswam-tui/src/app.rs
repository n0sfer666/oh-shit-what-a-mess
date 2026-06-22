use oswam_core::config::Theme;
use oswam_core::delete::Disposition;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanEntry, ScanResult};
use oswam_core::select::is_deletable;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Welcome,
    Scanning,
    Results,
}

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
    Enter,
    Cancel,
    Help,
    Theme,
    Group,
    Quit,
    Proceed,
}

#[derive(Debug, Clone, Default)]
pub struct ScanState {
    pub message: String,
    pub done: usize,
    pub total: usize,
    pub bytes: u64,
}

pub struct App {
    pub phase: Phase,
    pub scan: ScanState,
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
    pub start_scan_requested: bool,
    pub confirm_open: bool,
    pub confirm_choice: usize,
    pub decision: Option<Disposition>,
    pub(crate) first_run: bool,
}

impl App {
    pub fn new(theme: Theme, first_run: bool) -> Self {
        Self {
            phase: Phase::Welcome,
            scan: ScanState::default(),
            result: ScanResult {
                categories: Vec::new(),
                total_bytes: 0,
            },
            theme,
            panel: Panel::Sidebar,
            grouping: Grouping::Category,
            help_visible: false,
            help_scroll: 0,
            category_cursor: 0,
            file_cursor: 0,
            selected: HashSet::new(),
            should_quit: false,
            start_scan_requested: false,
            confirm_open: false,
            confirm_choice: 0,
            decision: None,
            first_run,
        }
    }

    pub fn set_result(&mut self, result: ScanResult) {
        self.selected.clear();
        for (ci, cat) in result.categories.iter().enumerate() {
            for (ei, entry) in cat.entries.iter().enumerate() {
                if is_deletable(entry) && entry.risk == RiskLevel::Safe {
                    self.selected.insert((ci, ei));
                }
            }
        }
        self.result = result;
        self.phase = Phase::Results;
        self.help_visible = self.first_run;
    }

    pub fn update_scan(&mut self, message: String, done: usize, total: usize, bytes: u64) {
        self.phase = Phase::Scanning;
        self.scan = ScanState {
            message,
            done,
            total,
            bytes,
        };
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

pub(crate) fn max_risk(result: &ScanResult, ci: usize) -> RiskLevel {
    result.categories[ci]
        .entries
        .iter()
        .map(|e| e.risk)
        .max()
        .unwrap_or(RiskLevel::Safe)
}
