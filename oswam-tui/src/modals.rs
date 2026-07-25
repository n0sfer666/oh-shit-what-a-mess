use crate::app::App;
use crate::panels::centered;
use crate::scroll::wrapped_rows;
use crate::theme::{palette, risk_color};
use oswam_core::format::human_bytes;
use oswam_core::risk::RiskLevel;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_done(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Готово ")
        .border_style(Style::default().fg(pal.accent));
    let s = app.summary.unwrap_or_default();
    let mut text = vec![
        Line::raw(""),
        Line::styled(
            format!("Обработано {} элементов", s.count),
            Style::default().fg(pal.accent),
        ),
    ];
    if s.freed > 0 {
        text.push(Line::raw(format!(
            "Освобождено сразу ~{}",
            human_bytes(s.freed)
        )));
    }
    if s.trashed > 0 {
        text.push(Line::raw(format!(
            "В Корзине ~{} — освободится после её очистки",
            human_bytes(s.trashed)
        )));
    }
    if s.partial > 0 {
        text.push(Line::styled(
            format!("Частично: {} — внутри осталось защищённое", s.partial),
            Style::default().fg(risk_color(RiskLevel::Caution)),
        ));
    }
    if s.untouched > 0 {
        text.push(Line::styled(
            format!(
                "Не тронуто: {} — всё содержимое защищено или недоступно",
                s.untouched
            ),
            Style::default().fg(risk_color(RiskLevel::Caution)),
        ));
    }
    if s.failed > 0 {
        text.push(Line::styled(
            format!("Не удалось: {} (нет доступа или занято)", s.failed),
            Style::default().fg(risk_color(RiskLevel::Danger)),
        ));
    }
    text.push(Line::raw(""));
    text.push(Line::raw("q — выход"));
    let popup = centered(area, 64, 12);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        popup,
    );
}

pub fn render_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Удалить выбранное? ")
        .border_style(Style::default().fg(pal.accent));
    let text = confirm_lines(app, pal.muted);
    let width = 56.min(area.width);
    let rows = wrapped_rows(&text, width.saturating_sub(2));
    let popup = centered(area, width, rows.saturating_add(2));
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn confirm_lines(app: &App, muted: Color) -> Vec<Line<'static>> {
    let opt = |idx: usize, label: String| {
        let mut style = Style::default();
        if app.confirm_choice == idx {
            style = style.add_modifier(Modifier::REVERSED);
        }
        Line::styled(format!("  {label}  "), style)
    };
    let (forced, forced_bytes) = app.selected_permanent_only();
    let trash_label = if forced > 0 {
        "В Корзину — что можно, остальное безвозвратно".to_string()
    } else {
        "В Корзину (можно восстановить)".to_string()
    };
    let unknown = app.selected_unknown_size();
    let unknown_note = if unknown > 0 {
        format!(" + {unknown} без оценки размера")
    } else {
        String::new()
    };
    let mut text = vec![
        Line::raw(format!(
            "Освободится ~{}{unknown_note}",
            human_bytes(app.selected_total_bytes())
        )),
        Line::raw(""),
        opt(0, trash_label),
        opt(1, "Безвозвратно удалить".to_string()),
    ];
    if forced > 0 {
        text.push(Line::styled(
            format!(
                "{forced} из выбранного (~{}) — только безвозвратно",
                human_bytes(forced_bytes)
            ),
            Style::default().fg(risk_color(RiskLevel::Danger)),
        ));
    }
    text.push(Line::styled(
        "Корзина освободит место только после её очистки.",
        Style::default().fg(muted),
    ));
    text.push(Line::styled(
        "Вне домашней папки и в Корзине — безвозвратно.",
        Style::default().fg(muted),
    ));
    text.push(Line::raw(""));
    text.push(Line::raw("↑↓ выбор · Enter подтвердить · Esc отмена"));
    text
}
