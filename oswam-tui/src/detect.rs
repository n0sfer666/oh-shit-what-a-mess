use oswam_core::config::Theme;

pub fn detect_theme(colorfgbg: Option<&str>) -> Theme {
    match colorfgbg.and_then(parse_bg) {
        Some(bg) if bg >= 7 => Theme::Light,
        _ => Theme::Dark,
    }
}

fn parse_bg(value: &str) -> Option<u8> {
    value.split(';').next_back()?.trim().parse().ok()
}

pub fn detect_from_env() -> Theme {
    detect_theme(std::env::var("COLORFGBG").ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_background() {
        assert_eq!(detect_theme(Some("15;0")), Theme::Dark);
    }

    #[test]
    fn light_background() {
        assert_eq!(detect_theme(Some("0;15")), Theme::Light);
    }

    #[test]
    fn missing_defaults_dark() {
        assert_eq!(detect_theme(None), Theme::Dark);
        assert_eq!(detect_theme(Some("garbage")), Theme::Dark);
    }
}
