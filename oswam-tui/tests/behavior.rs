use oswam_core::category::CleanupKind;
use oswam_core::config::Theme;
use oswam_core::delete::Disposition;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanCategory, ScanEntry, ScanResult};
use oswam_tui::app::{App, Grouping, Key, Panel, Phase};
use std::path::PathBuf;

fn entry(risk: RiskLevel, kind: CleanupKind, bytes: u64) -> ScanEntry {
    ScanEntry {
        display: "e".into(),
        path: PathBuf::from("/e"),
        kind,
        risk,
        physical_bytes: bytes,
        native: None,
    }
}

fn result() -> ScanResult {
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

fn results_app(first_run: bool) -> App {
    let mut a = App::new(Theme::Dark, first_run);
    a.set_result(result());
    a
}

#[test]
fn starts_in_welcome_phase() {
    let a = App::new(Theme::Dark, false);
    assert_eq!(a.phase, Phase::Welcome);
}

#[test]
fn welcome_enter_requests_scan() {
    let mut a = App::new(Theme::Dark, false);
    a.on_key(Key::Enter);
    assert!(a.start_scan_requested);
}

#[test]
fn update_scan_enters_scanning_phase() {
    let mut a = App::new(Theme::Dark, false);
    a.update_scan("x".into(), 1, 4, 10);
    assert_eq!(a.phase, Phase::Scanning);
    assert_eq!(a.scan.done, 1);
}

#[test]
fn set_result_enters_results_and_preselects_safe() {
    let a = results_app(false);
    assert_eq!(a.phase, Phase::Results);
    assert!(a.is_selected(0, 0));
    assert!(!a.is_selected(0, 1));
    assert!(a.is_selected(1, 0));
    assert_eq!(a.selected_total_bytes(), 100 + 9000);
}

#[test]
fn first_run_shows_help_after_results() {
    let a = results_app(true);
    assert!(a.help_visible);
}

#[test]
fn help_arrows_scroll_not_close() {
    let mut a = results_app(true);
    a.on_key(Key::Down);
    assert!(a.help_visible);
    assert_eq!(a.help_scroll, 1);
}

#[test]
fn navigation_clamps() {
    let mut a = results_app(false);
    a.on_key(Key::Up);
    assert_eq!(a.category_cursor, 0);
    a.on_key(Key::Bottom);
    assert_eq!(a.category_cursor, 1);
    a.on_key(Key::Down);
    assert_eq!(a.category_cursor, 1);
}

#[test]
fn table_toggle_selects_and_unselects() {
    let mut a = results_app(false);
    a.on_key(Key::Tab);
    a.on_key(Key::Tab);
    assert_eq!(a.panel, Panel::Table);
    a.on_key(Key::Down);
    a.on_key(Key::Space);
    assert!(a.is_selected(0, 1));
    a.on_key(Key::Space);
    assert!(!a.is_selected(0, 1));
}

#[test]
fn sidebar_space_selects_all_then_clears() {
    let mut a = results_app(false);
    a.on_key(Key::Space);
    assert!(a.is_selected(0, 0));
    assert!(a.is_selected(0, 1));
    a.on_key(Key::Space);
    assert!(!a.is_selected(0, 0));
    assert!(!a.is_selected(0, 1));
}

#[test]
fn grouping_by_size_reorders() {
    let mut a = results_app(false);
    assert_eq!(a.current_category(), Some(0));
    a.on_key(Key::Group);
    assert_eq!(a.grouping, Grouping::Size);
    assert_eq!(a.current_category(), Some(1));
}

#[test]
fn theme_toggle() {
    let mut a = results_app(false);
    a.on_key(Key::Theme);
    assert_eq!(a.theme, Theme::Light);
}

#[test]
fn proceed_opens_confirm_then_enter_decides() {
    let mut a = results_app(false);
    a.on_key(Key::Proceed);
    assert!(a.confirm_open);
    a.on_key(Key::Down);
    a.on_key(Key::Enter);
    assert_eq!(a.decision, Some(Disposition::Permanent));
}

#[test]
fn confirm_cancel_closes_modal() {
    let mut a = results_app(false);
    a.on_key(Key::Proceed);
    a.on_key(Key::Cancel);
    assert!(!a.confirm_open);
    assert_eq!(a.decision, None);
}

#[test]
fn quit_flag() {
    let mut a = results_app(false);
    a.on_key(Key::Quit);
    assert!(a.should_quit);
}
