use crate::app::{App, Phase};
use crate::modals::{render_confirm, render_done};
use crate::panels::{render_description, render_help, render_table};
use crate::screens::{render_deleting, render_scanning, render_welcome};
use crate::sidebar::render_sidebar;
use crate::theme::palette;
use oswam_core::format::human_bytes;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    match app.phase {
        Phase::Welcome => render_welcome(frame, app, frame.area()),
        Phase::Scanning => render_scanning(frame, app, frame.area()),
        Phase::Results => render_results(frame, app),
        Phase::Deleting => render_deleting(frame, app, frame.area()),
        Phase::Done => render_done(frame, app, frame.area()),
    }
}

fn render_results(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(rows[0]);
    let right = Layout::vertical([Constraint::Length(12), Constraint::Min(0)]).split(cols[1]);

    render_sidebar(frame, app, cols[0]);
    render_description(frame, app, right[0]);
    render_table(frame, app, right[1]);
    render_action_bar(frame, app, rows[1]);

    if app.confirm_open {
        render_confirm(frame, app, area);
    }
    if app.help_visible {
        render_help(frame, app, area);
    }
}

fn render_action_bar(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let unknown = app.selected_unknown_size();
    let unknown = if unknown > 0 {
        format!(" (+{unknown} ?)")
    } else {
        String::new()
    };
    let line = Line::from(vec![
        Span::styled(
            format!(
                " К удалению {}{unknown}",
                human_bytes(app.selected_total_bytes())
            ),
            Style::default().fg(pal.accent).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            " из {} │ Ctrl+P удалить · Space выбор · ? справка · a реком. · n снять · q выход",
            human_bytes(app.reclaimable_total())
        )),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().fg(pal.fg).bg(pal.bg)),
        area,
    );
}
