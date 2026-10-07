use crate::app::App;
use crate::describe::empty_note;
use crate::explain::blocked_text;
use crate::system_data_note::sudo_hint;
use crate::theme::{palette, risk_color, risk_symbol};
use oswam_core::format::human_bytes;
use oswam_core::risk::RiskLevel;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

pub fn category_lines(app: &App, ci: usize) -> Vec<Line<'static>> {
    let b = app.breakdown(ci);
    let muted = Style::default().fg(palette(app.theme).muted);
    let id = &app.result.categories[ci].id;
    let hint = sudo_hint(id, app.elevated).map(|t| Line::styled(t, muted));
    if b.shares.is_empty() && b.info_count == 0 && b.blocked_count == 0 {
        return std::iter::once(Line::raw(empty_note(id)))
            .chain(hint)
            .collect();
    }
    let unknown = if b.unknown > 0 {
        format!(" + {} без оценки размера", b.unknown)
    } else {
        String::new()
    };
    let mut lines = vec![Line::styled(
        format!(
            "Удалится: {} из {}{unknown}",
            human_bytes(b.picked_bytes),
            human_bytes(b.reclaimable)
        ),
        Style::default().add_modifier(Modifier::BOLD),
    )];
    for s in &b.shares {
        let manual = if s.manual > 0 {
            format!(" · {} только поштучно", s.manual)
        } else {
            String::new()
        };
        lines.push(Line::styled(
            format!(
                "{} {:<8}{:>9}  выбрано {}/{}{manual}",
                risk_symbol(s.risk),
                format!("{:?}", s.risk),
                human_bytes(s.bytes),
                s.picked,
                s.count
            ),
            Style::default().fg(risk_color(s.risk)),
        ));
    }
    if !b.blocked_apps.is_empty() {
        let app = b.blocked_apps.join(", ");
        lines.push(Line::styled(
            format!(
                "⏸ {} ({}) — {}",
                human_bytes(b.blocked_bytes),
                b.blocked_count,
                blocked_text(&app)
            ),
            Style::default().fg(risk_color(RiskLevel::Caution)),
        ));
    }
    if b.info_count > 0 {
        lines.push(Line::styled(
            format!(
                "ⓘ Только просмотр: {} ({})",
                human_bytes(b.info_bytes),
                b.info_count
            ),
            muted,
        ));
    }
    if b.shares.iter().any(|s| s.manual > 0) {
        lines.push(Line::styled(
            "Danger и «только безвозвратно» отмечаются по одному в «Файлах».",
            muted,
        ));
    }
    lines.extend(hint);
    lines.push(Line::styled(
        "Space — выбрать ✓ и ▲ · Enter — к файлам · a — рекомендованное",
        muted,
    ));
    lines
}
