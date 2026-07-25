mod support;

use oswam_tui::app::{Key, Panel};
use support::{drawn_text, render_once, results_app};

#[test]
fn help_arrows_scroll_not_close() {
    let mut a = results_app(true);
    a.on_key(Key::Down);
    assert!(a.help_visible);
    assert_eq!(a.help_scroll, 1);
}

#[test]
fn help_scroll_stops_at_the_last_line() {
    let mut a = results_app(true);
    for _ in 0..50 {
        a.on_key(Key::Down);
    }
    assert!(a.help_visible);
    assert!(a.help_scroll < 9, "{}", a.help_scroll);
}

#[test]
fn help_that_fits_the_rendered_area_does_not_scroll() {
    let mut a = results_app(true);
    render_once(&a, 80, 24);
    for _ in 0..50 {
        a.on_key(Key::Down);
    }
    assert_eq!(a.help_scroll, 0);
}

#[test]
fn description_scroll_limit_follows_the_rendered_width() {
    let scrolled = |width: u16| {
        let mut a = results_app(false);
        render_once(&a, width, 24);
        a.on_key(Key::Tab);
        for _ in 0..50 {
            a.on_key(Key::Down);
        }
        a.desc_scroll
    };
    assert_eq!(scrolled(120), 0);
    assert!(scrolled(46) > 0);
}

#[test]
fn description_scroll_stops_at_the_last_line() {
    let mut a = results_app(false);
    a.on_key(Key::Tab);
    assert_eq!(a.panel, Panel::Description);
    for _ in 0..50 {
        a.on_key(Key::Down);
    }
    assert!(a.desc_scroll < 6, "{}", a.desc_scroll);
}

#[test]
fn the_confirm_modal_keeps_every_warning_on_a_narrow_terminal() {
    let mut a = results_app(false);
    a.on_key(Key::Proceed);
    let text = drawn_text(&a, 44, 24);
    for word in ["очистки", "безвозвратно", "отмена"] {
        assert!(text.contains(word), "{word} missing:\n{text}");
    }
}
