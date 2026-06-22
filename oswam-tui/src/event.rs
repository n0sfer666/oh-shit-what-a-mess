use crate::app::Key;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn map_key(ev: KeyEvent) -> Option<Key> {
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    match (ev.code, ctrl) {
        (KeyCode::Char('p'), true) => Some(Key::Proceed),
        (KeyCode::Char('d'), true) => Some(Key::Down),
        (KeyCode::Char('u'), true) => Some(Key::Up),
        (KeyCode::Char('c'), true) => Some(Key::Quit),
        (KeyCode::Char('q'), false) => Some(Key::Quit),
        (KeyCode::Char('?'), false) => Some(Key::Help),
        (KeyCode::Char('t'), false) => Some(Key::Theme),
        (KeyCode::Char('o'), false) => Some(Key::Group),
        (KeyCode::Char(' '), false) => Some(Key::Space),
        (KeyCode::Tab, _) => Some(Key::Tab),
        (KeyCode::Char('j'), false) | (KeyCode::Down, _) => Some(Key::Down),
        (KeyCode::Char('k'), false) | (KeyCode::Up, _) => Some(Key::Up),
        (KeyCode::Char('h'), false) | (KeyCode::Left, _) => Some(Key::Left),
        (KeyCode::Char('l'), false) | (KeyCode::Right, _) => Some(Key::Right),
        (KeyCode::Char('g'), false) => Some(Key::Top),
        (KeyCode::Char('G'), false) => Some(Key::Bottom),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(code: KeyCode, ctrl: bool) -> KeyEvent {
        let mods = if ctrl {
            KeyModifiers::CONTROL
        } else {
            KeyModifiers::NONE
        };
        KeyEvent::new(code, mods)
    }

    #[test]
    fn maps_vim_and_arrows() {
        assert_eq!(map_key(ev(KeyCode::Char('j'), false)), Some(Key::Down));
        assert_eq!(map_key(ev(KeyCode::Down, false)), Some(Key::Down));
        assert_eq!(map_key(ev(KeyCode::Char('k'), false)), Some(Key::Up));
    }

    #[test]
    fn ctrl_p_proceeds_ctrl_c_quits() {
        assert_eq!(map_key(ev(KeyCode::Char('p'), true)), Some(Key::Proceed));
        assert_eq!(map_key(ev(KeyCode::Char('c'), true)), Some(Key::Quit));
    }

    #[test]
    fn plain_p_is_not_proceed() {
        assert_eq!(map_key(ev(KeyCode::Char('p'), false)), None);
    }

    #[test]
    fn unknown_is_none() {
        assert_eq!(map_key(ev(KeyCode::Char('z'), false)), None);
    }
}
