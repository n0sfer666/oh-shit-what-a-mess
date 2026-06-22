use crate::app::{App, Panel};
use crate::panels::{focus_block, render_description, render_help, render_table};
use crate::theme::{palette, risk_color, risk_symbol};
use oswam_core::format::human_bytes;
use oswam_core::select::is_deletable;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(rows[0]);
    let right = Layout::vertical([Constraint::Length(8), Constraint::Min(0)]).split(cols[1]);

    render_sidebar(frame, app, cols[0]);
    render_description(frame, app, right[0]);
    render_table(frame, app, right[1]);
    render_action_bar(frame, app, rows[1]);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use oswam_core::category::CleanupKind;
    use oswam_core::config::Theme;
    use oswam_core::risk::RiskLevel;
    use oswam_core::scan::{ScanCategory, ScanEntry, ScanResult};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;

    fn sample() -> ScanResult {
        ScanResult {
            categories: vec![
                ScanCategory {
                    id: "system".into(),
                    name: "Системный мусор".into(),
                    glyph: "C".into(),
                    entries: vec![ScanEntry {
                        display: "~/Library/Logs".into(),
                        path: PathBuf::from("/Users/x/Library/Logs"),
                        kind: CleanupKind::DeleteContents,
                        risk: RiskLevel::Safe,
                        physical_bytes: 1_500_000,
                        native: None,
                    }],
                    total_bytes: 1_500_000,
                },
                ScanCategory {
                    id: "big-data".into(),
                    name: "Большие данные".into(),
                    glyph: "B".into(),
                    entries: vec![ScanEntry {
                        display: "iOS backup".into(),
                        path: PathBuf::from("/Users/x/backup"),
                        kind: CleanupKind::InfoOnly,
                        risk: RiskLevel::Caution,
                        physical_bytes: 8_500_000_000,
                        native: None,
                    }],
                    total_bytes: 8_500_000_000,
                },
            ],
            total_bytes: 8_501_500_000,
        }
    }

    fn draw(app: &App) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        format!("{}", terminal.backend())
    }

    #[test]
    fn snapshot_main_layout_dark() {
        let app = App::new(sample(), Theme::Dark, false);
        insta::assert_snapshot!("main_layout_dark", draw(&app));
    }

    #[test]
    fn snapshot_help_overlay() {
        let app = App::new(sample(), Theme::Dark, true);
        insta::assert_snapshot!("help_overlay", draw(&app));
    }
}
