use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use std::cell::Cell;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Viewport {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Default)]
pub struct ScrollView(Cell<Viewport>);

impl ScrollView {
    pub fn observe(&self, inner: Rect) {
        self.0.set(Viewport {
            width: inner.width,
            height: inner.height,
        });
    }

    pub fn viewport(&self) -> Viewport {
        self.0.get()
    }

    pub fn max_scroll(&self, lines: &[Line<'_>]) -> u16 {
        let view = self.0.get();
        if view.width == 0 || view.height == 0 {
            return u16::try_from(lines.len().saturating_sub(1)).unwrap_or(u16::MAX);
        }
        wrapped_rows(lines, view.width).saturating_sub(view.height)
    }
}

pub fn wrapped_rows(lines: &[Line<'_>], width: u16) -> u16 {
    if width == 0 {
        return u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    let rows = Paragraph::new(lines.to_vec())
        .wrap(Wrap { trim: false })
        .line_count(width);
    u16::try_from(rows).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(width: u16, height: u16) -> ScrollView {
        let v = ScrollView::default();
        v.observe(Rect {
            x: 0,
            y: 0,
            width,
            height,
        });
        v
    }

    #[test]
    fn a_short_line_takes_one_row() {
        assert_eq!(wrapped_rows(&[Line::raw("abc")], 10), 1);
    }

    #[test]
    fn a_long_line_wraps_on_word_boundaries() {
        assert_eq!(wrapped_rows(&[Line::raw("aaa bbb ccc")], 7), 2);
    }

    #[test]
    fn a_word_longer_than_the_width_is_broken() {
        assert_eq!(wrapped_rows(&[Line::raw("aaaaaaaaa")], 3), 3);
    }

    #[test]
    fn wide_glyphs_count_as_two_cells() {
        assert_eq!(wrapped_rows(&[Line::raw("⛔ ⛔ ⛔")], 6), 2);
    }

    #[test]
    fn repeated_spaces_take_their_own_cells() {
        assert_eq!(wrapped_rows(&[Line::raw("aa  bb")], 5), 2);
    }

    #[test]
    fn the_limit_leaves_the_last_row_visible() {
        let lines = vec![Line::raw("aaa bbb ccc"), Line::raw("x"), Line::raw("y")];
        assert_eq!(view(7, 2).max_scroll(&lines), 2);
    }

    #[test]
    fn content_that_fits_does_not_scroll() {
        let lines = vec![Line::raw("x"), Line::raw("y")];
        assert_eq!(view(20, 5).max_scroll(&lines), 0);
    }

    #[test]
    fn without_a_rendered_area_the_limit_falls_back_to_line_count() {
        let lines = vec![Line::raw("x"), Line::raw("y"), Line::raw("z")];
        assert_eq!(ScrollView::default().max_scroll(&lines), 2);
    }
}
