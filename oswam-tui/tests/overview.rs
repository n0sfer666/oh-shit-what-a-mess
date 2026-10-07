mod support;

use oswam_core::category::CleanupKind;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::{ScanCategory, ScanResult};
use oswam_tui::app::{App, Key, Panel};
use support::{drawn_text, entry, results_app};

fn mixed_app() -> App {
    let mut forced = entry(RiskLevel::Safe, CleanupKind::DeletePath, 7);
    forced.permanent_only = true;
    let mut unknown = entry(RiskLevel::Safe, CleanupKind::NativeCommand, 0);
    unknown.size_unknown = true;
    let entries = vec![
        entry(RiskLevel::Safe, CleanupKind::DeletePath, 10),
        unknown,
        entry(RiskLevel::Danger, CleanupKind::DeletePath, 500),
        entry(RiskLevel::Caution, CleanupKind::DeletePath, 40),
        forced,
        entry(RiskLevel::Never, CleanupKind::InfoOnly, 900),
    ];
    let mut a = App::new(oswam_core::config::Theme::Dark, false, false);
    a.set_result(ScanResult {
        categories: vec![ScanCategory {
            id: "dev".into(),
            name: "Dev".into(),
            glyph: "d".into(),
            entries,
            total_bytes: 557,
        }],
        total_bytes: 557,
    });
    a
}

fn picked_sizes(a: &App) -> Vec<u64> {
    let mut sizes: Vec<u64> = a
        .selected_entries()
        .iter()
        .map(|e| e.physical_bytes)
        .collect();
    sizes.sort();
    sizes
}

#[test]
fn entries_are_listed_biggest_first_with_unknown_sizes_last() {
    let a = mixed_app();
    let sizes: Vec<(bool, u64)> = a.result.categories[0]
        .entries
        .iter()
        .map(|e| (e.size_unknown, e.physical_bytes))
        .collect();
    assert_eq!(
        sizes,
        vec![
            (false, 900),
            (false, 500),
            (false, 40),
            (false, 10),
            (false, 7),
            (true, 0)
        ]
    );
}

#[test]
fn the_sidebar_shows_what_will_go_out_of_what_could() {
    let text = drawn_text(&results_app(false), 80, 20);
    assert!(text.contains("100B / 150B"), "{text}");
}

#[test]
fn the_action_bar_leads_with_the_selected_total() {
    let text = drawn_text(&results_app(false), 80, 20);
    let bar = text.lines().last().unwrap_or_default();
    assert!(
        bar.trim_start().starts_with("К удалению 9.1 KB из "),
        "{bar}"
    );
}

#[test]
fn a_focused_category_explains_its_split_by_risk() {
    let text = drawn_text(&mixed_app(), 100, 24);
    assert!(text.contains("Удалится: 10 B из 557 B"), "{text}");
    assert!(text.contains("поштучно"), "{text}");
}

#[test]
fn space_on_a_category_never_sweeps_in_danger_or_permanent_only() {
    let mut a = mixed_app();
    a.on_key(Key::SelectNone);
    a.on_key(Key::Space);
    assert_eq!(picked_sizes(&a), vec![0, 10, 40]);
    a.on_key(Key::Space);
    assert!(a.selected.is_empty());
}

#[test]
fn a_restores_the_recommended_pick_and_n_clears_it() {
    let mut a = mixed_app();
    let recommended = picked_sizes(&a);
    assert_eq!(recommended, vec![0, 10]);
    a.on_key(Key::SelectNone);
    assert!(a.selected.is_empty());
    a.on_key(Key::SelectRecommended);
    assert_eq!(picked_sizes(&a), recommended);
}

#[test]
fn enter_dives_into_files_then_toggles_the_file() {
    let mut a = mixed_app();
    a.on_key(Key::Enter);
    assert_eq!(a.panel, Panel::Table);
    a.on_key(Key::Down);
    a.on_key(Key::Enter);
    assert!(picked_sizes(&a).contains(&500));
}

#[test]
fn space_off_keeps_files_picked_by_hand() {
    let mut a = mixed_app();
    a.on_key(Key::SelectNone);
    a.on_key(Key::Enter);
    a.on_key(Key::Down);
    a.on_key(Key::Enter);
    a.on_key(Key::Left);
    a.on_key(Key::Space);
    a.on_key(Key::Space);
    assert_eq!(picked_sizes(&a), vec![500]);
}

#[test]
fn space_clears_a_category_that_has_only_hand_picks() {
    let mut a = App::new(oswam_core::config::Theme::Dark, false, false);
    a.set_result(ScanResult {
        categories: vec![ScanCategory {
            id: "x".into(),
            name: "X".into(),
            glyph: "x".into(),
            entries: vec![entry(RiskLevel::Danger, CleanupKind::DeletePath, 5)],
            total_bytes: 5,
        }],
        total_bytes: 5,
    });
    a.selected.insert((0, 0));
    a.on_key(Key::Space);
    assert!(a.selected.is_empty());
}
