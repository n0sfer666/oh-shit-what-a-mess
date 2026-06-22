use oswam_core::config::Theme;
use oswam_core::delete::Disposition;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::ScanResult;
use oswam_core::select::is_deletable;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Welcome,
    Scanning,
    Results,
    Deleting,
    Done,
}

#[derive(Debug, Clone, Copy)]
pub struct Summary {
    pub count: usize,
    pub freed: u64,
    pub trashed: bool,
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
    pub delete: ScanState,
    pub summary: Option<Summary>,
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
    pub pending_delete: Option<Disposition>,
    pub(crate) first_run: bool,
}

impl App {
    pub fn new(theme: Theme, first_run: bool) -> Self {
        Self {
            phase: Phase::Welcome,
            scan: ScanState::default(),
            delete: ScanState::default(),
            summary: None,
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
            pending_delete: None,
            first_run,
        }
    }

    pub fn update_delete(&mut self, message: String, done: usize, total: usize, freed: u64) {
        self.phase = Phase::Deleting;
        self.delete = ScanState {
            message,
            done,
            total,
            bytes: freed,
        };
    }

    pub fn set_summary(&mut self, count: usize, freed: u64, trashed: bool) {
        self.phase = Phase::Done;
        self.summary = Some(Summary {
            count,
            freed,
            trashed,
        });
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
}

pub(crate) fn max_risk(result: &ScanResult, ci: usize) -> RiskLevel {
    result.categories[ci]
        .entries
        .iter()
        .map(|e| e.risk)
        .max()
        .unwrap_or(RiskLevel::Safe)
}
