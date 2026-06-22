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
                    path: PathBuf::from("/Users/x/Library/Logs"),
                    kind: CleanupKind::DeleteContents,
                    risk: RiskLevel::Safe,
                    physical_bytes: 1_500_000,
                    native: None,
                }],
                total_bytes: 1_500_000,
            },
            ScanCategory {
                id: "big-data".into(),
                name: "Большие данные".into(),
                glyph: "B".into(),
                entries: vec![ScanEntry {
                    display: "iOS backup".into(),
                    path: PathBuf::from("/Users/x/backup"),
                    kind: CleanupKind::InfoOnly,
                    risk: RiskLevel::Caution,
                    physical_bytes: 8_500_000_000,
                    native: None,
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
    let mut app = App::new(Theme::Dark, first_run);
    app.set_result(sample());
    app
}

#[test]
fn snapshot_welcome() {
    let app = App::new(Theme::Dark, false);
    insta::assert_snapshot!("welcome", draw(&app));
}

#[test]
fn snapshot_scanning() {
    let mut app = App::new(Theme::Dark, false);
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
