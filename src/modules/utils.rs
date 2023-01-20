use zellij_tile::prelude::*;
use zellij_tile::prelude::actions::Action;

pub fn filter_get_superkey(key: &Key, vac: &Vec<Action>) -> Option<&'static str> {
    match vac.first() {
        // No actions defined, ignore
        None => None,
        Some(action) => {
            // Ignore certain "default" keybindings that switch back to normal InputMode.
            // These include: ' ', '\n', 'Esc'
            if matches!(key, Key::Char(' ') | Key::Char('\n') | Key::Esc) {
                return None;
            }

            let is_mode_switch_or_quit_action = match action {
                // `SwitchToMode` action
                Action::SwitchToMode(mode) => {
                    matches!(
                        mode,
                        InputMode::Normal
                        | InputMode::Locked
                        | InputMode::Pane
                        | InputMode::Tab
                        | InputMode::Resize
                        | InputMode::Move
                        | InputMode::Search
                        | InputMode::Scroll
                        | InputMode::Session
                    )
                },
                // `Quit` action
                Action::Quit => true,
                // Not a `SwitchToMode` or `Quit` action, ignore
                _ => false,
            };

            if is_mode_switch_or_quit_action {
                return match key {
                    Key::Ctrl(_) => Some("Ctrl"),
                    Key::Alt(_) => Some("Alt"),
                    _ => None,
                };
            } else { None }
        },
    }
}

pub fn to_key(keybinds: &Vec<(Key, Vec<Action>)>, action: &[Action]) -> Option<Key> {
    keybinds.iter()
        // Get keybinds which match specified action
        .filter_map(|(key, actions)| {
            if actions.as_slice() == action {
                Some(*key)
            } else {
                None
            }
        })
        .into_iter()
        // Filter out certain "default" keybindings: ' ', '\n', 'Esc'
        .filter(|key| !matches!(key, Key::Char(' ') | Key::Char('\n') | Key::Esc))
        // Get only the first Key
        .next()
}