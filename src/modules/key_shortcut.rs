use std::str::FromStr;
use std::string::ToString;

use ansi_term::ANSIStrings;
use strum::{Display, EnumIter, EnumProperty, IntoEnumIterator};
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;

use super::colored_elements::ColoredElements;
use super::status_line::StatusLine;
use super::utils;

#[derive(Clone, Copy, Display, EnumIter, EnumProperty, PartialEq)]
pub enum KeyAction {
    #[strum(props(input_mode = "Locked"))]
    Lock,
    #[strum(props(input_mode = "Pane"))]
    Pane,
    #[strum(props(input_mode = "Tab"))]
    Tab,
    #[strum(props(input_mode = "Resize"))]
    Resize,
    #[strum(props(input_mode = "Move"))]
    Move,
    #[strum(props(input_mode = "Search"))]
    Search,
    #[strum(props(input_mode = "Scroll"))]
    Scroll,
    #[strum(props(input_mode = "Session"))]
    Session,
    #[strum(props(input_mode = "Tmux"))]
    Tmux,
    Quit,
}

impl KeyAction {
    fn input_mode(&self) -> &str {
        match self.get_str("input_mode") {
            Some(text) => text,
            None => "Normal",
        }
    }

    fn action(&self) -> Action {
        if *self == KeyAction::Quit {
            Action::Quit
        } else {
            match InputMode::from_str(self.input_mode()) {
                Ok(input_mode) => Action::SwitchToMode(input_mode),
                _ => Action::SwitchToMode(InputMode::Normal),
            }
        }
    }

    fn key_shortcut(&self, keybinds: &[(Key, Vec<Action>)], alternate: bool) -> KeyShortcut {
        KeyShortcut::new(
            // Unselect all initially by default
            if alternate { KeyMode::Unselected } else { KeyMode::UnselectedAlternate },
            *self,
            utils::to_key(keybinds, &[self.action()]),
        )
    }
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

    fn default_shortcuts(keybinds: &[(Key, Vec<Action>)]) -> Vec<Self> {
        // Unselect all by default
        KeyAction::iter()
            .enumerate()
            .map(|(i, key_action)| key_action.key_shortcut(keybinds, i % 2 == 0))
            .collect::<Vec<Self>>()
    }

    fn full_text(&self) -> String {
        self.action.to_string().to_uppercase()
    }

    fn letter_shortcut_and_count(&self, long: bool, with_prefix: bool) -> (String, usize) {
        let key = match self.key {
            Some(k) => k,
            None => Key::Null,
        };

        let key_binding = if with_prefix {
            format!("{key}")
        } else {
            match key {
                Key::F(c) => format!("{c}"),
                Key::Ctrl(c) => format!("{c}"),
                Key::Char(_) => format!("{key}"),
                Key::Alt(c) => format!("{c}"),
                _ => String::from("??"),
            }
        };
        let count = key_binding.chars().count();

        if long {
            (key_binding, count)
        } else {
            (format!(" {key_binding} "), count + 2)
        }
    }

    pub fn generate_status(
        &self, colored_elements: ColoredElements, separator: &str, long: bool, with_prefix: bool, first_tile: bool,
    ) -> StatusLine {
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
            let styled_text = colors.styled_text.paint(format!("{key_hint} "));

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

pub fn generate_shortcuts(keybinds: &[(Key, Vec<Action>)], mode: &InputMode) -> Vec<KeyShortcut> {
    let mut shortcuts = KeyShortcut::default_shortcuts(keybinds);

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