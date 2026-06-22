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
    progress_box(
        frame,
        app,
        area,
        " Сканирование… (q — прервать) ",
        &app.scan,
        "Найдено",
    );
}

pub fn render_deleting(frame: &mut Frame, app: &App, area: Rect) {
    progress_box(
        frame,
        app,
        area,
        " Удаление… (не закрывайте) ",
        &app.delete,
        "Освобождено",
    );
}

fn progress_box(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    title: &str,
    state: &crate::app::ScanState,
    bytes_label: &str,
) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title.to_string())
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
    frame.render_widget(Paragraph::new(state.message.clone()), rows[0]);
    let ratio = if state.total == 0 {
        0.0
    } else {
        state.done as f64 / state.total as f64
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(pal.accent))
            .ratio(ratio.clamp(0.0, 1.0))
            .label(format!("{}/{}", state.done, state.total)),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(format!("{bytes_label}: {}", human_bytes(state.bytes))),
        rows[2],
    );
}

pub fn render_done(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Готово ")
        .border_style(Style::default().fg(pal.accent));
    let s = app.summary;
    let (count, freed, trashed) = s
        .map(|s| (s.count, s.freed, s.trashed))
        .unwrap_or((0, 0, false));
    let note = if trashed {
        "Перемещено в Корзину — место освободится после её очистки."
    } else {
        "Удалено безвозвратно."
    };
    let text = vec![
        Line::raw(""),
        Line::styled(
            format!("Обработано {count} элементов"),
            Style::default().fg(pal.accent),
        ),
        Line::raw(format!("Освобождено ~{}", human_bytes(freed))),
        Line::raw(""),
        Line::raw(note),
        Line::raw(""),
        Line::raw("q — выход"),
    ];
    let popup = centered(area, 60, 11);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        popup,
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
