use crate::app::{max_risk, App, Panel};
use crate::breakdown::Breakdown;
use crate::panels::focus_block;
use crate::theme::{risk_color, risk_symbol};
use oswam_core::format::human_bytes;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{List, ListItem};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

pub fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Sidebar;
    let block = focus_block("Категории", focused, app);
    let width = usize::from(area.width.saturating_sub(2));
    let items: Vec<ListItem> = app
        .ordered_categories()
        .iter()
        .enumerate()
        .map(|(row, ci)| {
            let risk = max_risk(&app.result, *ci);
            let mut style = Style::default().fg(risk_color(risk));
            if focused && row == app.category_cursor {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let b = app.breakdown(*ci);
            let head = format!(
                "{} {} {} ",
                checkbox(&b),
                app.result.categories[*ci].glyph,
                risk_symbol(risk)
            );
            let line = row_text(&head, &app.result.categories[*ci].name, &sizes(&b), width);
            ListItem::new(line).style(style)
        })
        .collect();
    frame.render_widget(List::new(items).block(block), area);
}

pub fn compact_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0".into();
    }
    match human_bytes(bytes).split_once(' ') {
        Some((value, unit)) => format!("{value}{}", &unit[..1]),
        None => human_bytes(bytes),
    }
}

fn checkbox(b: &Breakdown) -> &'static str {
    match (b.count(), b.picked()) {
        (0, _) => "[-]",
        (_, 0) => "[ ]",
        (all, picked) if all == picked => "[x]",
        _ => "[~]",
    }
}

fn sizes(b: &Breakdown) -> String {
    if b.shares.is_empty() {
        let blocked = format!("⏸ {}", compact_bytes(b.blocked_bytes));
        let info = format!("ⓘ {}", compact_bytes(b.info_bytes));
        return match (b.blocked_count > 0, b.info_count > 0) {
            (true, true) => format!("{blocked} {info}"),
            (true, false) => blocked,
            _ => info,
        };
    }
    format!(
        "{} / {}",
        compact_bytes(b.picked_bytes),
        compact_bytes(b.reclaimable)
    )
}

fn row_text(head: &str, name: &str, right: &str, width: usize) -> String {
    let room = width.saturating_sub(head.width() + right.width() + 1);
    let name = fit(name, room);
    let pad = width
        .saturating_sub(head.width() + name.width() + right.width())
        .max(1);
    format!("{head}{name}{}{right}", " ".repeat(pad))
}

fn fit(text: &str, room: usize) -> String {
    if text.width() <= room {
        return text.to_string();
    }
    let mut out = String::new();
    for c in text.chars() {
        if out.width() + c.to_string().width() + 1 > room {
            break;
        }
        out.push(c);
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_sizes_drop_the_space_and_byte_suffix() {
        assert_eq!(compact_bytes(0), "0");
        assert_eq!(compact_bytes(512), "512B");
        assert_eq!(compact_bytes(8_100_000_000), "8.1G");
    }

    #[test]
    fn a_long_name_is_cut_so_the_sizes_stay_visible() {
        let row = row_text("[x] s ✓ ", "Очень длинное имя категории", "1.2G / 3.4G", 30);
        assert_eq!(row.width(), 30);
        assert!(row.ends_with("1.2G / 3.4G"), "{row}");
        assert!(row.contains('…'), "{row}");
    }
}
