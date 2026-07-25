use crate::app::{App, Panel};
use crate::describe::{empty_note, what_is_it, why_text};
use crate::explain::{action_text, select_hint, size_line};
use crate::theme::{palette, risk_color, risk_symbol};
use oswam_core::scan::ScanEntry;
use oswam_core::select::is_deletable;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;

pub fn focus_block(title: &str, focused: bool, app: &App) -> Block<'static> {
    let pal = palette(app.theme);
    let color = if focused { pal.focus } else { pal.muted };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(format!(" {title} "))
        .title_style(Style::default().fg(pal.accent))
}

pub fn render_description(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Description;
    let block = focus_block("Описание", focused, app);
    app.desc_view.observe(block.inner(area));
    let lines = description_lines(app);
    let offset = app.desc_scroll.min(app.desc_view.max_scroll(&lines));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((offset, 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub fn description_lines(app: &App) -> Vec<Line<'static>> {
    let category_id = app
        .current_category()
        .map(|ci| app.result.categories[ci].id.as_str())
        .unwrap_or("");
    match current_entry(app) {
        Some(entry) => vec![
            Line::from(vec![
                Span::styled(
                    risk_symbol(entry.risk),
                    Style::default().fg(risk_color(entry.risk)),
                ),
                Span::raw(format!(" {:?}", entry.risk)),
            ]),
            Line::styled(
                size_line(entry),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Line::raw(what_is_it(category_id, entry)),
            Line::raw(why_text(entry.risk)),
            Line::raw(action_text(entry)),
            Line::styled(
                select_hint(entry),
                Style::default().fg(palette(app.theme).muted),
            ),
        ],
        None => vec![Line::raw(empty_note(category_id, app.elevated))],
    }
}

pub fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Table;
    let block = focus_block("Файлы", focused, app);
    let rows: Vec<Row> = match app.current_category() {
        Some(ci) => app.result.categories[ci]
            .entries
            .iter()
            .enumerate()
            .map(|(ei, e)| {
                let mark = if !is_deletable(e) {
                    " ⓘ "
                } else if app.is_selected(ci, ei) {
                    "[x]"
                } else {
                    "[ ]"
                };
                let mut style = Style::default().fg(risk_color(e.risk));
                if focused && ei == app.file_cursor {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                Row::new(vec![
                    format!("{mark} {} {}", risk_symbol(e.risk), e.display),
                    e.size_label(),
                ])
                .style(style)
            })
            .collect(),
        None => Vec::new(),
    };
    let widths = [
        ratatui::layout::Constraint::Min(10),
        ratatui::layout::Constraint::Length(10),
    ];
    let mut state = TableState::default();
    if !rows.is_empty() {
        state.select(Some(app.file_cursor.min(rows.len() - 1)));
    }
    frame.render_stateful_widget(Table::new(rows, widths).block(block), area, &mut state);
}

pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Справка (любая клавиша — закрыть, ↑↓hjkl — скролл) ")
        .border_style(Style::default().fg(pal.accent));
    let popup = centered(area, 70, 12);
    app.help_view.observe(block.inner(popup));
    let lines = help_lines();
    let offset = app.help_scroll.min(app.help_view.max_scroll(&lines));
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((offset, 0))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

pub fn help_lines() -> Vec<Line<'static>> {
    vec![
        Line::raw("Навигация:  j/k ↑/↓  ·  h/l панели  ·  Tab  ·  g/G"),
        Line::raw("Space      выбрать категорию/файл"),
        Line::raw("j/k в «Описании» — прокрутка текста"),
        Line::raw("o          группировка (категория/размер/риск)"),
        Line::raw("t          тема (тёмная/светлая)"),
        Line::raw("Ctrl+P     удалить выбранное (Корзина/безвозвратно)"),
        Line::raw("?          справка   ·   q  выход"),
        Line::raw(""),
        Line::raw("Цвет + символ риска: ✓ Safe · ▲ Caution · ✗ Danger · ⛔ Never"),
    ]
}

fn current_entry(app: &App) -> Option<&ScanEntry> {
    let ci = app.current_category()?;
    app.result.categories[ci].entries.get(app.file_cursor)
}

pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}
