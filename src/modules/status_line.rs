use std::fmt::{Display, Error, Formatter};

use ansi_term::ANSIStrings;
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;

use super::colored_elements::ColoredElements;
use super::key_shortcut::{self, KeyShortcut};
use super::utils;

static MORE_MSG: &str = " ... ";
static MINIMAL_LENGTH_FOR_FULL_SHORTCUT_PRINTING: usize = 136;

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
    fn superkey(
        keybinds: &[(Key, Vec<Action>)], colored_elements: &ColoredElements, separator: &str, simplified_ui: bool,
    ) -> StatusLine {
        let mut superkeys = keybinds
            .iter()
            // Keep only `SwitchToMode` and `Quit` key-action entries and map to its superkey
            .filter_map(utils::filter_get_superkey);

        match superkeys.next() {
            // Check if all superkeys are the same, if keys exist
            Some(superkey) if superkeys.all(|str| str == superkey) => {
                let prefix_text = format!(" {superkey} +{}", if simplified_ui { " " } else { "" });

                let prefix = colored_elements.superkey_prefix.paint(&prefix_text);
                let suffix_separator = colored_elements.superkey_suffix_separator.paint(separator);

                StatusLine {
                    part: ANSIStrings(&[prefix, suffix_separator]).to_string(),
                    len: prefix_text.chars().count() // Superkey
                        + separator.chars().count(), // Separator
                }
            }
            // Otherwise, don't print superkey
            _ => StatusLine::default(),
        }
    }

    fn shortcuts(&mut self, shortcuts: Vec<KeyShortcut>, colored_elements: &ColoredElements, separator: &str, max_len: usize) {
        let shared_super = self.len > 0;
        let mut line_empty = self.len == 0;

        for shortcut in shortcuts {
            // Build up StatusLine one shortcut at a time
            let shortcut_status = shortcut.generate_status(
                colored_elements,
                separator,
                max_len > MINIMAL_LENGTH_FOR_FULL_SHORTCUT_PRINTING,
                shared_super,
                line_empty,
            );

            // Append to self
            self.part = format!("{}{}", self.part, shortcut_status.part);
            self.len += shortcut_status.len;
            line_empty = line_empty && self.len == 0;
        }
    }

    fn add_shortcut_keybindings(&mut self, colored_elements: &ColoredElements, text: &str, keys: &[Key], is_locked_mode: bool) {
        if keys.is_empty() && !is_locked_mode { return; }

        let separator = if self.len == 0 { " " } else { " / " };
        let mut bits = vec![colored_elements.text.paint(separator)];
        bits.extend(colored_elements.paint_keys(keys));
        bits.push(colored_elements.text.bold().paint(format!(" {text}")));
        let part = ANSIStrings(&bits);

        self.part = format!("{}{}", self.part, part);
        self.len += ansi_term::unstyled_len(&part);
    }

    fn nonstandard_mode_hints(&mut self, mode_info: &ModeInfo, colored_elements: &ColoredElements, max_len: usize) {
        let keys_and_hints = utils::get_keys_and_hints(mode_info);
        let more_msg = colored_elements.text.paint(MORE_MSG);
        let is_locked_mode = mode_info.mode == InputMode::Locked;

        let mut full_hints = StatusLine::default();
        let mut short_hints = StatusLine::default();
        let mut is_full_overflowing = false;
        for (long, short, keys) in keys_and_hints {
            if !is_full_overflowing {
                // Build the full version as long as it fits
                full_hints.add_shortcut_keybindings(colored_elements, &long, &keys, is_locked_mode);
                is_full_overflowing = self.len + full_hints.len > max_len;
            }

            if self.len + short_hints.len + 6 + keys.len() + short.chars().count() + MORE_MSG.chars().count() > max_len {
                // StatusLine is long enough, finishing
                self.part = format!("{}{}{}", self.part, short_hints.part, more_msg);
                self.len += short_hints.len + MORE_MSG.chars().count();
                return;
            }
            // Build the short version of StatusLine
            short_hints.add_shortcut_keybindings(colored_elements, &short, &keys, is_locked_mode);
        }

        // Return the full version if possible, otherwise return the short version
        let actual_hints = if is_full_overflowing { short_hints } else { full_hints };
        self.part = format!("{}{}", self.part, actual_hints.part);
        self.len += actual_hints.len;
    }

    fn fill(&mut self, colored_elements: &ColoredElements) {
        self.part = format!("{}{}", self.part, colored_elements.filler.paint("\u{1b}[0K"));
    }

    pub fn build(
        mode_info: &ModeInfo, keybinds: &[(Key, Vec<Action>)], colored_elements: &ColoredElements, simplified_ui: bool,
        separator: &str, max_len: usize,
    ) -> StatusLine {
        // Initial StatusLine with superkey indicator
        let mut status = Self::superkey(keybinds, colored_elements, separator, simplified_ui);

        // Append shortcuts to status
        let shortcuts = key_shortcut::generate_shortcuts(keybinds, &mode_info.mode);
        status.shortcuts(shortcuts, colored_elements, separator, max_len);

        // Append key bindings and hints for each nonstandard modes
        status.nonstandard_mode_hints(mode_info, colored_elements, max_len);

        // Fill the rest of the line
        status.fill(colored_elements);

        status
    }
}
