use crate::app::App;
use crate::panels::centered;
use crate::theme::palette;
use oswam_core::format::human_bytes;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};
use ratatui::Frame;

pub fn render_welcome(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" OSWaM ")
        .border_style(Style::default().fg(pal.accent));
    let text = vec![
        Line::raw(""),
        Line::raw("OSWaM — безопасная очистка места на macOS"),
        Line::raw(""),
        Line::raw("Найдём кэши, dev-артефакты, Docker, логи и корзину,"),
        Line::raw("покажем физически освобождаемое место."),
        Line::raw(""),
        Line::styled(
            "Enter — просканировать систему",
            Style::default().fg(pal.accent),
        ),
        Line::raw("q — выход"),
    ];
    let popup = centered(area, 60, 12);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        popup,
    );
}

pub fn render_scanning(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Сканирование… (q — прервать) ")
        .border_style(Style::default().fg(pal.accent));
    let popup = centered(area, 64, 8);
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);
    frame.render_widget(Paragraph::new(app.scan.message.clone()), rows[0]);
    let ratio = if app.scan.total == 0 {
        0.0
    } else {
        app.scan.done as f64 / app.scan.total as f64
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(pal.accent))
            .ratio(ratio.clamp(0.0, 1.0))
            .label(format!("{}/{}", app.scan.done, app.scan.total)),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(format!("Найдено: {}", human_bytes(app.scan.bytes))),
        rows[2],
    );
}

pub fn render_confirm(frame: &mut Frame, app: &App, popup: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Удалить выбранное? ")
        .border_style(Style::default().fg(pal.accent));
    let opt = |idx: usize, label: &str| {
        let mut style = Style::default();
        if app.confirm_choice == idx {
            style = style.add_modifier(Modifier::REVERSED);
        }
        Line::styled(format!("  {label}  "), style)
    };
    let text = vec![
        Line::raw(format!(
            "Освободится ~{}",
            human_bytes(app.selected_total_bytes())
        )),
        Line::raw(""),
        opt(0, "В Корзину (можно восстановить)"),
        opt(1, "Безвозвратно удалить"),
        Line::raw(""),
        Line::raw("↑↓ выбор · Enter подтвердить · Esc отмена"),
    ];
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        popup,
    );
}
