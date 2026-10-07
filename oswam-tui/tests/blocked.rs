mod support;

use oswam_core::category::CleanupKind;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanCategory, ScanResult};
use oswam_tui::app::App;
use oswam_tui::overview::category_lines;
use support::entry;

fn claude_app(blocked: bool) -> App {
    let mut image = entry(RiskLevel::Caution, CleanupKind::DeletePath, 4096);
    image.blocking_app = Some("Claude".into());
    image.blocked = blocked;
    let mut a = App::new(oswam_core::config::Theme::Dark, false, false);
    a.set_result(ScanResult {
        categories: vec![ScanCategory {
            id: "claude".into(),
            name: "Claude Desktop".into(),
            glyph: "c".into(),
            entries: vec![image, entry(RiskLevel::Never, CleanupKind::InfoOnly, 512)],
            total_bytes: 4096,
        }],
        total_bytes: 4096,
    });
    a
}

fn overview(a: &App) -> String {
    category_lines(a, 0)
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_blocked_entry_is_counted_apart_from_view_only_ones() {
    let a = claude_app(true);
    let b = a.breakdown(0);
    assert_eq!((b.blocked_count, b.blocked_bytes), (1, 4096));
    assert_eq!((b.info_count, b.info_bytes), (1, 512));
    assert_eq!(b.reclaimable, 0);
    let text = overview(&a);
    assert!(text.contains("закройте"), "{text}");
    assert!(text.contains("Claude"), "{text}");
}

#[test]
fn a_closed_app_makes_the_entry_reclaimable_again() {
    let a = claude_app(false);
    let b = a.breakdown(0);
    assert_eq!(b.blocked_count, 0);
    assert_eq!(b.reclaimable, 4096);
    assert!(!overview(&a).contains("закройте"));
}

#[test]
fn an_app_reopened_before_the_delete_is_reported_in_the_summary() {
    let mut a = claude_app(false);
    a.set_summary(oswam_tui::app::Summary {
        blocked: 1,
        ..Default::default()
    });
    let out = support::drawn_text(&a, 100, 30);
    assert!(out.contains("Пропущено: 1"), "{out}");
}

#[test]
fn the_sidebar_shows_blocked_and_view_only_bytes_side_by_side() {
    let out = support::drawn_text(&claude_app(true), 120, 30);
    assert!(out.contains("⏸ 4"), "{out}");
    assert!(out.contains("ⓘ 512"), "{out}");
}

#[test]
fn several_apps_are_all_named_in_the_overview() {
    let mut a = claude_app(true);
    let mut other = entry(RiskLevel::Caution, CleanupKind::DeletePath, 8);
    other.blocking_app = Some("Docker".into());
    other.blocked = true;
    a.result.categories[0].entries.push(other);
    assert_eq!(a.breakdown(0).blocked_apps, vec!["Claude", "Docker"]);
    assert!(overview(&a).contains("Claude, Docker"));
}
