use oswam_core::config::Theme;
use oswam_core::risk::RiskLevel;
use ratatui::style::Color;

pub fn risk_symbol(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => "✓",
        RiskLevel::Caution => "▲",
        RiskLevel::Danger => "✗",
        RiskLevel::Never => "⛔",
    }
}

pub fn risk_color(risk: RiskLevel) -> Color {
    match risk {
        RiskLevel::Safe => Color::Green,
        RiskLevel::Caution => Color::Yellow,
        RiskLevel::Danger => Color::Red,
        RiskLevel::Never => Color::DarkGray,
    }
}

pub struct Palette {
    pub fg: Color,
    pub bg: Color,
    pub accent: Color,
    pub muted: Color,
    pub focus: Color,
}

pub fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Dark => Palette {
            fg: Color::Gray,
            bg: Color::Black,
            accent: Color::Cyan,
            muted: Color::DarkGray,
            focus: Color::Cyan,
        },
        Theme::Light => Palette {
            fg: Color::Black,
            bg: Color::White,
            accent: Color::Blue,
            muted: Color::Gray,
            focus: Color::Blue,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_distinct_per_risk() {
        let all = [
            RiskLevel::Safe,
            RiskLevel::Caution,
            RiskLevel::Danger,
            RiskLevel::Never,
        ];
        let symbols: Vec<&str> = all.iter().map(|r| risk_symbol(*r)).collect();
        let unique: std::collections::HashSet<_> = symbols.iter().collect();
        assert_eq!(unique.len(), 4);
    }

    #[test]
    fn colors_differ_between_themes() {
        assert_ne!(palette(Theme::Dark).bg, palette(Theme::Light).bg);
        assert_ne!(palette(Theme::Dark).fg, palette(Theme::Light).fg);
    }

    #[test]
    fn safe_is_green() {
        assert_eq!(risk_color(RiskLevel::Safe), Color::Green);
    }
}
