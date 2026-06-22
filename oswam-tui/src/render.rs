use crate::app::{App, Panel, Phase};
use crate::panels::{centered, focus_block, render_description, render_help, render_table};
use crate::screens::{render_confirm, render_scanning, render_welcome};
use crate::theme::{palette, risk_color, risk_symbol};
use oswam_core::format::human_bytes;
use oswam_core::select::is_deletable;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    match app.phase {
        Phase::Welcome => render_welcome(frame, app, frame.area()),
        Phase::Scanning => render_scanning(frame, app, frame.area()),
        Phase::Results => render_results(frame, app),
    }
}

fn render_results(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(rows[0]);
    let right = Layout::vertical([Constraint::Length(8), Constraint::Min(0)]).split(cols[1]);

    render_sidebar(frame, app, cols[0]);
    render_description(frame, app, right[0]);
    render_table(frame, app, right[1]);
    render_action_bar(frame, app, rows[1]);

    if app.confirm_open {
        render_confirm(frame, app, centered(area, 56, 9));
    }
    if app.help_visible {
        render_help(frame, app, area);
    }
}

fn category_checkbox(app: &App, ci: usize) -> &'static str {
    let entries = &app.result.categories[ci].entries;
    let deletable: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| is_deletable(e))
        .map(|(ei, _)| ei)
        .collect();
    if deletable.is_empty() {
        return "[-]";
    }
    let selected = deletable
        .iter()
        .filter(|ei| app.is_selected(ci, **ei))
        .count();
    if selected == 0 {
        "[ ]"
    } else if selected == deletable.len() {
        "[x]"
    } else {
        "[~]"
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Sidebar;
    let block = focus_block("Категории", focused, app);
    let order = app.ordered_categories();
    let items: Vec<ListItem> = order
        .iter()
        .enumerate()
        .map(|(row, ci)| {
            let cat = &app.result.categories[*ci];
            let risk = crate::app::max_risk(&app.result, *ci);
            let mut style = Style::default().fg(risk_color(risk));
            if focused && row == app.category_cursor {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let line = Line::from(vec![Span::styled(
                format!(
                    "{} {} {} {}  {}",
                    category_checkbox(app, *ci),
                    cat.glyph,
                    risk_symbol(risk),
                    cat.name,
                    human_bytes(cat.total_bytes),
                ),
                style,
            )]);
            ListItem::new(line)
        })
        .collect();
    frame.render_widget(List::new(items).block(block), area);
}

fn render_action_bar(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let text = format!(
        " Ctrl+P: удалить · Space: выбор · o: группировка · t: тема · ?: справка · q: выход   Выбрано: {}",
        human_bytes(app.selected_total_bytes())
    );
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(pal.fg).bg(pal.bg)),
        area,
    );
}
