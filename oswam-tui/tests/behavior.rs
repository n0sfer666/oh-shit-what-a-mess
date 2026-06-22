use oswam_core::category::CleanupKind;
use oswam_core::config::Theme;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanCategory, ScanEntry, ScanResult};
use oswam_tui::app::{App, Grouping, Key, Panel};
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

fn app() -> App {
    App::new(result(), Theme::Dark, false)
}

#[test]
fn preselects_only_safe() {
    let a = app();
    assert!(a.is_selected(0, 0));
    assert!(!a.is_selected(0, 1));
    assert!(a.is_selected(1, 0));
    assert_eq!(a.selected_total_bytes(), 100 + 9000);
}

#[test]
fn first_run_shows_help_and_any_key_closes() {
    let mut a = App::new(result(), Theme::Dark, true);
    assert!(a.help_visible);
    a.on_key(Key::Tab);
    assert!(!a.help_visible);
}

#[test]
fn help_arrows_scroll_not_close() {
    let mut a = App::new(result(), Theme::Dark, true);
    a.on_key(Key::Down);
    assert!(a.help_visible);
    assert_eq!(a.help_scroll, 1);
    a.on_key(Key::Up);
    assert_eq!(a.help_scroll, 0);
}

#[test]
fn navigation_clamps() {
    let mut a = app();
    a.on_key(Key::Up);
    assert_eq!(a.category_cursor, 0);
    a.on_key(Key::Bottom);
    assert_eq!(a.category_cursor, 1);
    a.on_key(Key::Down);
    assert_eq!(a.category_cursor, 1);
}

#[test]
fn table_toggle_selects_and_unselects() {
    let mut a = app();
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
    let mut a = app();
    a.on_key(Key::Space);
    assert!(a.is_selected(0, 0));
    assert!(a.is_selected(0, 1));
    a.on_key(Key::Space);
    assert!(!a.is_selected(0, 0));
    assert!(!a.is_selected(0, 1));
}

#[test]
fn grouping_by_size_reorders() {
    let mut a = app();
    assert_eq!(a.current_category(), Some(0));
    a.on_key(Key::Group);
    assert_eq!(a.grouping, Grouping::Size);
    assert_eq!(a.current_category(), Some(1));
}

#[test]
fn theme_toggle() {
    let mut a = app();
    a.on_key(Key::Theme);
    assert_eq!(a.theme, Theme::Light);
}

#[test]
fn proceed_and_quit_flags() {
    let mut a = app();
    a.on_key(Key::Proceed);
    assert!(a.proceed_requested);
    a.on_key(Key::Quit);
    assert!(a.should_quit);
}
