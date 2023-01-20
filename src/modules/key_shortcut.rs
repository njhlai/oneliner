use ansi_term::ANSIStrings;
use zellij_tile::prelude::*;
use zellij_tile::prelude::actions::Action;

use super::status_line::StatusLine;
use super::colored_elements::ColoredElements;
use super::utils;

#[derive(PartialEq)]
pub enum KeyAction {
    Lock,
    Pane,
    Tab,
    Resize,
    Move,
    Search,
    Scroll,
    Session,
    Tmux,
    Quit,
}

pub enum KeyMode {
    Unselected,
    UnselectedAlternate,
    Selected,
    Disabled,
}

pub struct KeyShortcut {
    pub mode: KeyMode,
    pub action: KeyAction,
    pub key: Option<Key>,
}

impl KeyShortcut {
    fn new(mode: KeyMode, action: KeyAction, key: Option<Key>) -> Self {
        KeyShortcut { mode, action, key }
    }

    fn full_text(&self) -> String {
        match self.action {
            KeyAction::Lock => String::from("LOCK"),
            KeyAction::Pane => String::from("PANE"),
            KeyAction::Tab => String::from("TAB"),
            KeyAction::Resize => String::from("RESIZE"),
            KeyAction::Move => String::from("MOVE"),
            KeyAction::Search => String::from("SEARCH"),
            KeyAction::Scroll => String::from("SCROLL"),
            KeyAction::Session => String::from("SESSION"),
            KeyAction::Tmux => String::from("TMUX"),
            KeyAction::Quit => String::from("QUIT"),
        }
    }

    fn letter_shortcut_and_count(&self, long: bool, with_prefix: bool) -> (String, usize) {
        let key = match self.key {
            Some(k) => k,
            None => Key::Null,
        };

        let key_binding = if with_prefix {
            format!("{}", key)
        } else {
            match key {
                Key::F(c) => format!("{}", c),
                Key::Ctrl(c) => format!("{}", c),
                Key::Char(_) => format!("{}", key),
                Key::Alt(c) => format!("{}", c),
                _ => String::from("??"),
            }
        };
        let count = key_binding.chars().count();

        if long {
            (key_binding, count)
        } else {
            (format!(" {} ", key_binding), count + 2)
        }
    }

    pub fn generate_status(&self, colored_elements: ColoredElements, separator: &str, long: bool, with_prefix: bool, first_tile: bool) -> StatusLine {
        let key_hint = self.full_text();
        let (key_binding, count) = match (&self.mode, &self.key) {
            // Disabled or unreachable mode, don't print
            (_, None) | (KeyMode::Disabled, _) => return StatusLine::default(),
            // Reachable mode, print
            (_, Some(_)) => self.letter_shortcut_and_count(long, !with_prefix),
        };

        let colors = match self.mode {
            KeyMode::Unselected => colored_elements.unselected,
            KeyMode::UnselectedAlternate => colored_elements.unselected_alternate,
            KeyMode::Selected => colored_elements.selected,
            KeyMode::Disabled => colored_elements.disabled,
        };
        let start_separator = if !with_prefix && first_tile { "" } else { separator };
        let prefix_separator = colors.prefix_separator.paint(start_separator);
        let char_shortcut = colors.char_shortcut.paint(key_binding);
        let suffix_separator = colors.suffix_separator.paint(separator);

        if long {
            let char_left_separator = colors.char_left_separator.paint(" <".to_string());
            let char_right_separator = colors.char_right_separator.paint("> ".to_string());
            let styled_text = colors.styled_text.paint(format!("{} ", key_hint));

            // Full form printing
            StatusLine {
                part: ANSIStrings(&[
                    prefix_separator,
                    char_left_separator,
                    char_shortcut,
                    char_right_separator,
                    styled_text,
                    suffix_separator,
                ])
                .to_string(),
                len: start_separator.chars().count() // Separator
                    + 2                              // " <"
                    + count                          // Key binding
                    + 2                              // "> "
                    + key_hint.chars().count()       // Key hint (mode)
                    + 1                              // " "
                    + separator.chars().count(),     // Separator
            }
        } else {
            // Short form printing
            StatusLine {
                part: ANSIStrings(&[prefix_separator, char_shortcut, suffix_separator]).to_string(),
                len: separator.chars().count()      // Separator
                    + count                         // Key binding
                    + separator.chars().count(),    // Separator
            }
        }
    }
}

pub fn generate_shortcuts(keybinds: &Vec<(Key, Vec<Action>)>, mode: &InputMode) -> Vec<KeyShortcut> {
    // Unselect all by default
    let mut shortcuts = vec![
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Lock,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Locked)]),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Pane,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Pane)]),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Tab,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Tab)]),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Resize,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Resize)]),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Move,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Move)]),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Search,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Search)]),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Scroll,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Scroll)]),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Session,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Session)]),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Tmux,
            utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Tmux)]),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Quit,
            utils::to_key(keybinds, &[Action::Quit]),
        ),
    ];

    let key_action = match mode {
        // Return on Normal mode
        InputMode::Normal | InputMode::Prompt => return shortcuts,
        // Otherwise, proceed with modifying shortcuts
        InputMode::Locked => KeyAction::Lock,
        InputMode::Pane | InputMode::RenamePane => KeyAction::Pane,
        InputMode::Tab | InputMode::RenameTab => KeyAction::Tab,
        InputMode::Resize => KeyAction::Resize,
        InputMode::Move => KeyAction::Move,
        InputMode::Search | InputMode::EnterSearch => KeyAction::Search,
        InputMode::Scroll => KeyAction::Scroll,
        InputMode::Session => KeyAction::Session,
        InputMode::Tmux => KeyAction::Tmux,
    };

    for shortcut in shortcuts.iter_mut() {
        if shortcut.action == key_action {
            // Highlight current mode
            shortcut.mode = KeyMode::Selected;
            shortcut.key = utils::to_key(keybinds, &[Action::SwitchToMode(InputMode::Normal)]);
        } else {
            // Hide all other modes
            shortcut.mode = KeyMode::Disabled;
        }
    }

    shortcuts
}