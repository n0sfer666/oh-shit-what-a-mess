use oswam_core::category::CleanupKind;
use oswam_core::config::Theme;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanCategory, ScanEntry, ScanResult};
use oswam_tui::app::{App, Key};
use oswam_tui::render::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

fn sample() -> ScanResult {
    ScanResult {
        categories: vec![
            ScanCategory {
                id: "system".into(),
                name: "Системный мусор".into(),
                glyph: "C".into(),
                entries: vec![ScanEntry {
                    display: "~/Library/Logs".into(),
                    path: PathBuf::from("/Users/tester/Library/Logs"),
                    kind: CleanupKind::DeleteContents,
                    risk: RiskLevel::Safe,
                    physical_bytes: 1_500_000,
                    native: None,
                    companions: Vec::new(),
                    shared_bytes: 0,
                    size_unknown: false,
                    partial: false,
                    needs_root: false,
                    permanent_only: false,
                }],
                total_bytes: 1_500_000,
            },
            ScanCategory {
                id: "big-data".into(),
                name: "Большие данные".into(),
                glyph: "B".into(),
                entries: vec![ScanEntry {
                    display: "iOS backup".into(),
                    path: PathBuf::from("/Users/tester/backup"),
                    kind: CleanupKind::InfoOnly,
                    risk: RiskLevel::Caution,
                    physical_bytes: 8_500_000_000,
                    native: None,
                    companions: Vec::new(),
                    shared_bytes: 0,
                    size_unknown: false,
                    partial: false,
                    needs_root: false,
                    permanent_only: false,
                }],
                total_bytes: 8_500_000_000,
            },
        ],
        total_bytes: 8_501_500_000,
    }
}

fn draw(app: &App) -> String {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(f, app)).unwrap();
    format!("{}", terminal.backend())
}

fn results_app(first_run: bool) -> App {
    let mut app = App::new(Theme::Dark, first_run, false);
    app.set_result(sample());
    app
}

#[test]
fn snapshot_welcome() {
    let app = App::new(Theme::Dark, false, false);
    insta::assert_snapshot!("welcome", draw(&app));
}

#[test]
fn snapshot_scanning() {
    let mut app = App::new(Theme::Dark, false, false);
    app.update_scan(
        "Сканирую ~/Library/Caches/Yarn".into(),
        6,
        16,
        9_200_000_000,
    );
    insta::assert_snapshot!("scanning", draw(&app));
}

#[test]
fn snapshot_main_layout_dark() {
    insta::assert_snapshot!("main_layout_dark", draw(&results_app(false)));
}

#[test]
fn snapshot_help_overlay() {
    insta::assert_snapshot!("help_overlay", draw(&results_app(true)));
}

#[test]
fn snapshot_confirm_modal() {
    let mut app = results_app(false);
    app.on_key(Key::Proceed);
    insta::assert_snapshot!("confirm_modal", draw(&app));
}

fn big_result(n: usize) -> ScanResult {
    let entries: Vec<ScanEntry> = (0..n)
        .map(|i| ScanEntry {
            display: format!("entry-{i:02}"),
            path: PathBuf::from(format!("/Users/tester/{i}")),
            kind: CleanupKind::DeletePath,
            risk: RiskLevel::Caution,
            physical_bytes: 1_000_000,
            native: None,
            companions: Vec::new(),
            shared_bytes: 0,
            size_unknown: false,
            partial: false,
            needs_root: false,
            permanent_only: false,
        })
        .collect();
    ScanResult {
        total_bytes: entries.len() as u64 * 1_000_000,
        categories: vec![ScanCategory {
            id: "projects".into(),
            name: "Проекты".into(),
            glyph: "P".into(),
            entries,
            total_bytes: n as u64 * 1_000_000,
        }],
    }
}

#[test]
fn table_scrolls_to_keep_cursor_visible() {
    let mut app = App::new(Theme::Dark, false, false);
    app.set_result(big_result(40));
    app.on_key(Key::Right);
    app.on_key(Key::Bottom);
    let out = draw(&app);
    assert!(
        out.contains("entry-39"),
        "cursor row must be visible:\n{out}"
    );
    assert!(
        !out.contains("entry-00"),
        "top row must scroll out of view:\n{out}"
    );
}

#[test]
fn info_only_entry_is_marked_and_unselectable() {
    let mut app = results_app(false);
    app.on_key(Key::Down);
    let out = draw(&app);
    assert!(out.contains("ⓘ"), "info marker must show:\n{out}");
    assert!(
        out.contains("удаление недоступно"),
        "description must explain:\n{out}"
    );
    app.on_key(Key::Right);
    let before = app.selected_total_bytes();
    app.on_key(Key::Space);
    assert_eq!(
        app.selected_total_bytes(),
        before,
        "pressing Space on info-only entry must not change selection"
    );
}

#[test]
fn description_shows_freed_size_for_deletable() {
    let app = results_app(false);
    let out = draw(&app);
    assert!(
        out.contains("Освободится: 1.5 MB"),
        "freed size must show:\n{out}"
    );
}
