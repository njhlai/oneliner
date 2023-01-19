use std::fmt::{Display, Error, Formatter};

use ansi_term::ANSIStrings;
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;

use super::colored_elements::ColoredElements;
use super::key_shortcut::{KeyAction, KeyMode, KeyShortcut};

#[derive(Default)]
pub struct StatusLine {
    pub part: String,
    pub len: usize,
}

impl Display for StatusLine {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.part)
    }
}

impl StatusLine {
    fn superkey(keybinds: &Vec<(Key, Vec<Action>)>, colored_elements: ColoredElements, separator: &str, arrow_fonts: bool) -> StatusLine {
        let mut keyvec = keybinds
            .iter()
            .filter_map(|(key, vac)| key_actions_filter(key, vac));

        let prefix_text = match keyvec.next() {
            Some(modifier) => {
                // check if all modifiers are the same
                if keyvec.all(|str| str == modifier) {
                    if arrow_fonts {
                        // Add extra space in simplified ui
                        format!(" {} + ", modifier.to_string())
                    } else {
                        format!(" {} +", modifier.to_string())
                    }
                } else {
                    return StatusLine::default()
                }
            },
            _ => return StatusLine::default(),
        };

        let prefix = colored_elements.superkey_prefix.paint(&prefix_text);
        let suffix_separator = colored_elements.superkey_suffix_separator.paint(separator);

        StatusLine {
            part: ANSIStrings(&[prefix, suffix_separator]).to_string(),
            len: prefix_text.chars().count() + separator.chars().count(),
        }
    }

    pub fn build(mode_info: &ModeInfo, max_len: usize, separator: &str) -> StatusLine {
        let keybinds = &(mode_info.get_mode_keybinds());
        let colored_elements = ColoredElements::color_elements(&(mode_info.style.colors), !mode_info.capabilities.arrow_fonts);

        let mut status = Self::superkey(keybinds, colored_elements, separator, mode_info.capabilities.arrow_fonts);

        // Unselect all by default
        let mut default_keys = vec![
            KeyShortcut::new(
                KeyMode::Unselected,
                KeyAction::Lock,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Locked)]),
            ),
            KeyShortcut::new(
                KeyMode::UnselectedAlternate,
                KeyAction::Pane,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Pane)]),
            ),
            KeyShortcut::new(
                KeyMode::Unselected,
                KeyAction::Tab,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Tab)]),
            ),
            KeyShortcut::new(
                KeyMode::UnselectedAlternate,
                KeyAction::Resize,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Resize)]),
            ),
            KeyShortcut::new(
                KeyMode::Unselected,
                KeyAction::Move,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Move)]),
            ),
            KeyShortcut::new(
                KeyMode::UnselectedAlternate,
                KeyAction::Search,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Search)]),
            ),
            KeyShortcut::new(
                KeyMode::UnselectedAlternate,
                KeyAction::Scroll,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Scroll)]),
            ),
            KeyShortcut::new(
                KeyMode::Unselected,
                KeyAction::Session,
                to_char(keybinds, &[Action::SwitchToMode(InputMode::Session)]),
            ),
            KeyShortcut::new(
                KeyMode::UnselectedAlternate,
                KeyAction::Quit,
                to_char(keybinds, &[Action::Quit]),
            ),
        ];

        if let Some(key_shortcut) = get_key_shortcut_for_mode(&mut default_keys, &mode_info.mode) {
            key_shortcut.mode = KeyMode::Selected;
            key_shortcut.key = to_char(keybinds, &[Action::SwitchToMode(InputMode::Normal)]);
        }

        let shared_super = status.len > 0;
        for key in default_keys {
            let line_empty = status.len == 0;
            let key_status = key.shortcut(colored_elements, separator, max_len > 110, shared_super, line_empty);
            status.part = format!("{}{}", status.part, key_status.part);
            status.len += key_status.len;
        }

        status
    }
}

fn key_actions_filter(key: &Key, vac: &Vec<Action>) -> Option<&'static str> {
    match vac.first() {
        // No actions defined, ignore
        None => None,
        Some(vac) => {
            // We ignore certain "default" keybindings that switch back to normal InputMode.
            // These include: ' ', '\n', 'Esc'
            if matches!(key, Key::Char(' ') | Key::Char('\n') | Key::Esc) {
                return None;
            }

            // `SwitchToMode` action
            if let Action::SwitchToMode(mode) = vac {
                return match mode {
                    // Store the keys that switch to displayed modes
                    InputMode::Normal
                    | InputMode::Locked
                    | InputMode::Pane
                    | InputMode::Tab
                    | InputMode::Resize
                    | InputMode::Move
                    | InputMode::Search
                    | InputMode::Scroll
                    | InputMode::Session => {
                        match key {
                            Key::Ctrl(_) => Some("Ctrl"),
                            Key::Alt(_) => Some("Alt"),
                            _ => return None,
                        }
                    },
                    _ => None,
                };
            }

            // `Quit` action
            if let actions::Action::Quit = vac {
                return match key {
                    Key::Ctrl(_) => Some("Ctrl"),
                    Key::Alt(_) => Some("Alt"),
                    _ => return None,
                };
            }

            // Not a `SwitchToMode` or `Quit` action, ignore
            None
        },
    }
}

fn to_char(keybinds: &Vec<(Key, Vec<Action>)>, action: &[Action]) -> Option<Key> {
    keybinds.iter()
        .filter_map(|(key, actions)| {
            if actions.as_slice() == action {
                Some(*key)
            } else {
                None
            }
        })
        .into_iter()
        .filter(|key| !matches!(key, Key::Char('\n') | Key::Char(' ') | Key::Esc))
        .next()
}

fn get_key_shortcut_for_mode<'a>(shortcuts: &'a mut [KeyShortcut], mode: &InputMode) -> Option<&'a mut KeyShortcut> {
    let key_action = match mode {
        InputMode::Normal | InputMode::Prompt | InputMode::Tmux => return None,
        InputMode::Locked => KeyAction::Lock,
        InputMode::Pane | InputMode::RenamePane => KeyAction::Pane,
        InputMode::Tab | InputMode::RenameTab => KeyAction::Tab,
        InputMode::Resize => KeyAction::Resize,
        InputMode::Move => KeyAction::Move,
        InputMode::Search | InputMode::EnterSearch => KeyAction::Search,
        InputMode::Scroll => KeyAction::Scroll,
        InputMode::Session => KeyAction::Session,
    };

    let mut val = None;
    for shortcut in shortcuts.iter_mut() {
        if shortcut.action == key_action {
            val = Some(shortcut);
        } else {
            // hide all other modes
            shortcut.mode = KeyMode::Disabled;
        }
    }
    val
}