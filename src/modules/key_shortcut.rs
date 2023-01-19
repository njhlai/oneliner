use ansi_term::ANSIStrings;
use zellij_tile::prelude::*;

use super::status_line::StatusLine;
use super::colored_elements::ColoredElements;

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
    pub fn new(mode: KeyMode, action: KeyAction, key: Option<Key>) -> Self {
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
            KeyAction::Quit => String::from("QUIT"),
        }
    }

    fn letter_shortcut(&self, with_prefix: bool) -> String {
        let key = match self.key {
            Some(k) => k,
            None => return String::from("?"),
        };
        if with_prefix {
            format!("{}", key)
        } else {
            match key {
                Key::F(c) => format!("{}", c),
                Key::Ctrl(c) => format!("{}", c),
                Key::Char(_) => format!("{}", key),
                Key::Alt(c) => format!("{}", c),
                _ => String::from("??"),
            }
        }
    }

    pub fn shortcut(&self, colored_elements: ColoredElements, separator: &str, long: bool, with_prefix: bool, first_tile: bool) -> StatusLine {
        let key_hint = self.full_text();
        let key_binding = match (&self.mode, &self.key) {
            (_, None) | (KeyMode::Disabled, _) => return StatusLine::default(),
            (_, Some(_)) => self.letter_shortcut(!with_prefix),
        };

        let colors = match self.mode {
            KeyMode::Unselected => colored_elements.unselected,
            KeyMode::UnselectedAlternate => colored_elements.unselected_alternate,
            KeyMode::Selected => colored_elements.selected,
            KeyMode::Disabled => colored_elements.disabled,
        };
        let start_separator = if !with_prefix && first_tile { "" } else { separator };
        let prefix_separator = colors.prefix_separator.paint(start_separator);
        let char_left_separator = colors.char_left_separator.paint(" <".to_string());
        let char_shortcut = if long {
            colors.char_shortcut.paint(key_binding.to_string())
        } else {
            colors.char_shortcut.paint(format!(" {} ", key_binding))
        };
        let char_right_separator = colors.char_right_separator.paint("> ".to_string());
        let styled_text = colors.styled_text.paint(format!("{} ", key_hint));
        let suffix_separator = colors.suffix_separator.paint(separator);

        if long {
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
                    + key_binding.chars().count()    // Key binding
                    + 2                              // "> "
                    + key_hint.chars().count()       // Key hint (mode)
                    + 1                              // " "
                    + separator.chars().count(),     // Separator
            }
        } else {
            StatusLine {
                part: ANSIStrings(&[prefix_separator, char_shortcut, suffix_separator]).to_string(),
                len: separator.chars().count()      // Separator
                    + 1                             // " "
                    + key_binding.chars().count()   // Key binding
                    + 1                             // " "
                    + separator.chars().count(),    // Separator
            }
        }
    }
}