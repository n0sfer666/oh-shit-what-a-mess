#![allow(dead_code)]

use oswam_core::category::CleanupKind;
use oswam_core::config::Theme;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanCategory, ScanEntry, ScanResult};
use oswam_tui::app::App;
use oswam_tui::render::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

pub fn entry(risk: RiskLevel, kind: CleanupKind, bytes: u64) -> ScanEntry {
    ScanEntry {
        display: "e".into(),
        path: PathBuf::from("/e"),
        kind,
        risk,
        physical_bytes: bytes,
        native: None,
        companions: Vec::new(),
        shared_bytes: 0,
        size_unknown: false,
        partial: false,
        needs_root: false,
        permanent_only: false,
    }
}

pub fn result() -> ScanResult {
    ScanResult {
        categories: vec![
            ScanCategory {
                id: "system".into(),
                name: "S".into(),
                glyph: "s".into(),
                entries: vec![
                    entry(RiskLevel::Safe, CleanupKind::DeleteContents, 100),
                    entry(RiskLevel::Caution, CleanupKind::DeleteContents, 50),
                ],
                total_bytes: 150,
            },
            ScanCategory {
                id: "dev".into(),
                name: "D".into(),
                glyph: "d".into(),
                entries: vec![entry(RiskLevel::Safe, CleanupKind::DeletePath, 9000)],
                total_bytes: 9000,
            },
        ],
        total_bytes: 9150,
    }
}

pub fn results_app(first_run: bool) -> App {
    let mut a = App::new(Theme::Dark, first_run, false);
    a.set_result(result());
    a
}

pub fn render_once(app: &App, width: u16, height: u16) {
    let mut term = Terminal::new(TestBackend::new(width, height)).expect("terminal");
    term.draw(|f| render(f, app)).expect("draw");
}

pub fn drawn_text(app: &App, width: u16, height: u16) -> String {
    let mut term = Terminal::new(TestBackend::new(width, height)).expect("terminal");
    term.draw(|f| render(f, app)).expect("draw");
    let buffer = term.backend().buffer().clone();
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
