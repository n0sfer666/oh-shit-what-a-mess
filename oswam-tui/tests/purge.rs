mod support;

use oswam_core::delete::Trashed;
use oswam_tui::app::{App, Key, Phase, Summary};
use std::path::PathBuf;
use support::{drawn_text, results_app};

fn trashed() -> Vec<Trashed> {
    vec![Trashed {
        path: PathBuf::from("/Users/tester/.Trash/x"),
        bytes: 5000,
        ino: 1,
        dev: 1,
    }]
}

fn done_app(items: Vec<Trashed>) -> App {
    let mut a = results_app(false);
    a.trashed = items;
    a.set_summary(Summary {
        count: 1,
        trashed: 5000,
        ..Summary::default()
    });
    a
}

#[test]
fn e_on_the_summary_asks_to_empty_what_oswam_trashed() {
    let mut a = done_app(trashed());
    assert!(drawn_text(&a, 80, 20).contains("e — "));
    a.on_key(Key::Purge);
    assert!(a.purge_requested);
}

#[test]
fn e_does_nothing_when_oswam_trashed_nothing() {
    let mut a = done_app(Vec::new());
    assert!(!drawn_text(&a, 80, 20).contains("e — "));
    a.on_key(Key::Purge);
    assert!(!a.purge_requested);
}

#[test]
fn an_emptied_trash_moves_bytes_from_pending_to_freed() {
    let mut a = done_app(trashed());
    a.set_purged(Summary {
        count: 1,
        freed: 5000,
        ..Summary::default()
    });
    assert_eq!(a.phase, Phase::Done);
    let s = a.summary.unwrap_or_default();
    assert_eq!((s.freed, s.trashed), (5000, 0));
    assert!(a.trashed.is_empty());
}

#[test]
fn an_emptied_trash_reports_what_it_left_behind() {
    let mut a = done_app(trashed());
    a.set_purged(Summary {
        untouched: 1,
        failed: 2,
        ..Summary::default()
    });
    let s = a.summary.unwrap_or_default();
    assert_eq!((s.kept_in_trash, s.failed, s.trashed), (1, 2, 5000));
    let text = drawn_text(&a, 80, 24);
    assert!(text.contains("Остались в Корзине: 1"), "{text}");
    assert!(text.contains("Не удалось: 2"), "{text}");
}
