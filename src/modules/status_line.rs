use std::fmt::{Display, Error, Formatter};

use ansi_term::ANSIStrings;
use zellij_tile::prelude::*;
use zellij_tile::prelude::actions::Action;

use super::colored_elements::ColoredElements;
use super::key_shortcut::{self, KeyShortcut};
use super::utils;

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
        let mut superkeys = keybinds
            .iter()
            // Keep only `SwitchToMode` and `Quit` key-action entries and map to its superkey
            .filter_map(|(key, vac)| utils::filter_get_superkey(key, vac));

        let prefix_text = match superkeys.next() {
            // Check if all superkeys are the same, if keys exist
            Some(superkey) if superkeys.all(|str| str == superkey) => {
                if arrow_fonts {
                    // Add extra space in simplified ui
                    format!(" {} + ", superkey.to_string())
                } else {
                    format!(" {} +", superkey.to_string())
                }
            },
            // Otherwise, don't print superkey
            _ => return StatusLine::default(),
        };

        let prefix = colored_elements.superkey_prefix.paint(&prefix_text);
        let suffix_separator = colored_elements.superkey_suffix_separator.paint(separator);

        StatusLine {
            part: ANSIStrings(&[prefix, suffix_separator]).to_string(),
            len: prefix_text.chars().count() // Superkey
                + separator.chars().count(), // Separator
        }
    }

    fn shortcuts(&mut self, shortcuts: Vec<KeyShortcut>, colored_elements: ColoredElements, separator: &str, max_len: usize) {
        let shared_super = self.len > 0;
        let mut line_empty = self.len == 0;

        for shortcut in shortcuts {
            // Build up StatusLine one shortcut at a time
            let shortcut_status = shortcut.generate_status(colored_elements, separator, max_len > 110, shared_super, line_empty);

            // Append to self
            self.part = format!("{}{}", self.part, shortcut_status.part);
            self.len += shortcut_status.len;
            if line_empty { line_empty = self.len == 0; }
        }
    }

    pub fn build(mode: &InputMode, keybinds: &Vec<(Key, Vec<Action>)>, colored_elements: ColoredElements, arrow_fonts: bool, separator: &str, max_len: usize) -> StatusLine {
        // Initial StatusLine with superkey indicator
        let mut status = Self::superkey(keybinds, colored_elements, separator, arrow_fonts);

        let shortcuts = key_shortcut::generate_shortcuts(keybinds, mode);
        // Append shortcuts to status
        status.shortcuts(shortcuts, colored_elements, separator, max_len);

        status
    }
}