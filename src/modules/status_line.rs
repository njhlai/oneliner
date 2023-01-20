use std::fmt::{Display, Error, Formatter};

use ansi_term::ANSIStrings;
use zellij_tile::prelude::*;
use zellij_tile::prelude::actions::Action;

use super::colored_elements::ColoredElements;
use super::key_shortcut::{self, KeyShortcut};
use super::utils;

static MORE_MSG: &str = " ... ";

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

    fn add_shortcut_keybindings(&mut self, mode_info: &ModeInfo, text: &str, keys: Vec<Key>) {
        let shortcut = full_length_shortcut(self.len == 0, keys, text, mode_info.style.colors);

        self.part = format!("{}{}", self.part, shortcut);
        self.len += shortcut.len;
    }

    fn shortcut_list_nonstandard_mode(&mut self, mode_info: &ModeInfo, max_len: usize) {
        let keys_and_hints = utils::get_keys_and_hints(mode_info);

        let palette = mode_info.style.colors;
        let bg_color = match palette.theme_hue {
            ThemeHue::Dark => palette.black,
            ThemeHue::Light => palette.white,
        };
        let text_color = match palette.theme_hue {
            ThemeHue::Dark => palette.white,
            ThemeHue::Light => palette.black,
        };
        let more_msg = style!(text_color, bg_color).paint(MORE_MSG);

        let mut full_ver = StatusLine::default();
        let mut short_ver = StatusLine::default();
        let mut is_full_overflowing = false;
        for (long, short, keys) in keys_and_hints.into_iter() {
            if !is_full_overflowing {
                full_ver.add_shortcut_keybindings(mode_info, &long, keys.clone());
                is_full_overflowing = self.len + full_ver.len > max_len;
            }

            if self.len + short_ver.len + MORE_MSG.chars().count() > max_len {
                self.part = format!("{}{}{}", self.part, short_ver.part, more_msg);
                self.len += short_ver.len + MORE_MSG.chars().count();
                return;
            }
            short_ver.add_shortcut_keybindings(mode_info, &short, keys);
        }

        let actual_ver = if !is_full_overflowing { full_ver } else { short_ver };
        self.part = format!("{}{}", self.part, actual_ver.part);
        self.len += actual_ver.len;
    }

    pub fn build(mode_info: &ModeInfo, keybinds: &Vec<(Key, Vec<Action>)>, colored_elements: ColoredElements, arrow_fonts: bool, separator: &str, max_len: usize) -> StatusLine {
        // Initial StatusLine with superkey indicator
        let mut status = Self::superkey(keybinds, colored_elements, separator, arrow_fonts);

        // Append shortcuts to status
        let shortcuts = key_shortcut::generate_shortcuts(keybinds, &mode_info.mode);
        status.shortcuts(shortcuts, colored_elements, separator, max_len);

        // Append key binding for each non-normal, non-locked modes
        status.shortcut_list_nonstandard_mode(mode_info, max_len);

        status
    }
}

use ansi_term::{unstyled_len, ANSIString, Style};
use zellij_tile_utils::style;

pub fn style_key_with_modifier(keyvec: &[Key], palette: &Palette) -> Vec<ANSIString<'static>> {
    // Nothing to do, quit...
    if keyvec.is_empty() { return vec![]; }

    let bg_color = match palette.theme_hue {
        ThemeHue::Dark => palette.black,
        ThemeHue::Light => palette.white,
    };
    let text_color = match palette.theme_hue {
        ThemeHue::Dark => palette.white,
        ThemeHue::Light => palette.black,
    };
    let key_color = palette.green;
    let modifier_color = palette.orange;
    let mut ret = vec![];

    let mut keyvec_iter = keyvec.iter();
    let maybe_modifier = match keyvec_iter.next() {
        Some(key) if keyvec_iter.all(|str| str == key) => {
            match key {
                Key::Ctrl(_) => Some("Ctrl".to_string()),
                Key::Alt(_) => Some("Alt".to_string()),
                _ => None,
            }
        },
        _ => None,
    };

    // Prints modifier key
    let modifier_str = match maybe_modifier {
        Some(modifier) => modifier,
        None => "".to_string(),
    };
    let no_modifier = modifier_str.is_empty();
    let painted_modifier = if modifier_str.is_empty() {
        Style::new().paint("")
    } else {
        style!(modifier_color, bg_color).bold().paint(modifier_str)
    };
    ret.push(painted_modifier);

    // Prints key group start
    let group_start_str = if no_modifier { "<" } else { " + <" };
    ret.push(style!(text_color, bg_color).paint(group_start_str));

    // Prints the keys
    let key = keyvec
        .iter()
        .map(|key| {
            if no_modifier {
                format!("{}", key)
            } else {
                match key {
                    Key::Ctrl(c) => format!("{}", Key::Char(*c)),
                    Key::Alt(c) => format!("{}", c),
                    _ => format!("{}", key),
                }
            }
        })
        .collect::<Vec<String>>();

    // Special handling of some pre-defined keygroups
    let key_string = key.join("");
    let key_separator = match &key_string[..] {
        "HJKL" => "",
        "hjkl" => "",
        "←↓↑→" => "",
        "←→" => "",
        "↓↑" => "",
        _ => "|",
    };

    for (idx, key) in key.iter().enumerate() {
        if idx > 0 && !key_separator.is_empty() {
            ret.push(style!(text_color, bg_color).paint(key_separator));
        }
        ret.push(style!(key_color, bg_color).bold().paint(key.clone()));
    }

    let group_end_str = ">";
    ret.push(style!(text_color, bg_color).paint(group_end_str));

    ret
}

fn full_length_shortcut(is_first_shortcut: bool, key: Vec<Key>, action: &str, palette: Palette) -> StatusLine {
    if key.is_empty() { return StatusLine::default(); }

    let bg_color = match palette.theme_hue {
        ThemeHue::Dark => palette.black,
        ThemeHue::Light => palette.white,
    };
    let text_color = match palette.theme_hue {
        ThemeHue::Dark => palette.white,
        ThemeHue::Light => palette.black,
    };

    let separator = if is_first_shortcut { " " } else { " / " };
    let mut bits: Vec<ANSIString> = vec![style!(text_color, bg_color).paint(separator)];
    bits.extend(style_key_with_modifier(&key, &palette));
    bits.push(style!(text_color, bg_color).bold().paint(format!(" {}", action)));
    let part = ANSIStrings(&bits);

    StatusLine {
        part: part.to_string(),
        len: unstyled_len(&part),
    }
}