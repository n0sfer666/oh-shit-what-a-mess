use crate::app::{App, Panel};
use crate::theme::{palette, risk_color, risk_symbol};
use oswam_core::category::CleanupKind;
use oswam_core::format::human_bytes;
use oswam_core::risk::RiskLevel;
use oswam_core::scan::ScanEntry;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap};
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

fn action_text(entry: &ScanEntry) -> String {
    match entry.kind {
        CleanupKind::DeleteContents => "Удалит содержимое (Корзина/безвозвратно по выбору)".into(),
        CleanupKind::DeletePath => "Удалит каталог целиком".into(),
        CleanupKind::NativeCommand => match &entry.native {
            Some(spec) => format!("Выполнит: {}", spec.clean.join(" ")),
            None => "Нативная команда".into(),
        },
        CleanupKind::InfoOnly => "Только информация — удаление недоступно".into(),
    }
}

fn why_text(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => "Безопасно: регенерируемый кэш, приложение пересоздаст.",
        RiskLevel::Caution => "Осторожно: возможно используется процессом или частично нужное.",
        RiskLevel::Danger => "Опасно: удаление может повлиять на работу.",
        RiskLevel::Never => "Никогда: системное/защищённое, удаление заблокировано.",
    }
}

pub fn render_description(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Description;
    let block = focus_block("Описание", focused, app);
    let lines = match current_entry(app) {
        Some(entry) => vec![
            Line::from(vec![
                Span::styled(
                    risk_symbol(entry.risk),
                    Style::default().fg(risk_color(entry.risk)),
                ),
                Span::raw(format!(" {:?}", entry.risk)),
            ]),
            Line::raw(why_text(entry.risk)),
            Line::raw(""),
            Line::raw(action_text(entry)),
        ],
        None => vec![Line::raw("Нет данных для отображения.")],
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
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
                let mark = if app.is_selected(ci, ei) {
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
                    human_bytes(e.physical_bytes),
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
    frame.render_widget(Table::new(rows, widths).block(block), area);
}

pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let pal = palette(app.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Справка (любая клавиша — закрыть, ↑↓hjkl — скролл) ")
        .border_style(Style::default().fg(pal.accent));
    let text = vec![
        Line::raw("Навигация:  j/k ↑/↓  ·  h/l панели  ·  Tab  ·  g/G  ·  Ctrl+d/Ctrl+u"),
        Line::raw("Space      выбрать категорию/файл"),
        Line::raw("o          группировка (категория/размер/риск)"),
        Line::raw("t          тема (тёмная/светлая)"),
        Line::raw("Ctrl+P     удалить выбранное (Корзина/безвозвратно)"),
        Line::raw("?          справка   ·   q  выход"),
        Line::raw(""),
        Line::raw("Цвет + символ риска: ✓ Safe · ▲ Caution · ✗ Danger · ⛔ Never"),
    ];
    let popup = centered(area, 70, 12);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .scroll((app.help_scroll, 0))
            .wrap(Wrap { trim: false }),
        popup,
    );
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
