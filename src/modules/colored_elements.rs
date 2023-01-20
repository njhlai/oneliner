use ansi_term::{ANSIString, Style};
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

#[derive(Clone, Copy)]
pub struct ColoredElements {
    pub selected: SegmentStyle,
    pub unselected: SegmentStyle,
    pub unselected_alternate: SegmentStyle,
    pub disabled: SegmentStyle,
    // superkey
    pub superkey_prefix: Style,
    pub superkey_suffix_separator: Style,
    // status
    pub modifier: Style,
    pub key: Style,
    pub text: Style,
}

#[derive(Clone, Copy)]
pub struct SegmentStyle {
    pub prefix_separator: Style,
    pub char_left_separator: Style,
    pub char_shortcut: Style,
    pub char_right_separator: Style,
    pub styled_text: Style,
    pub suffix_separator: Style,
}

impl ColoredElements {
    pub fn color_elements(palette: &Palette, different_color_alternates: bool) -> ColoredElements {
        let background = match palette.theme_hue {
            ThemeHue::Dark => palette.black,
            ThemeHue::Light => palette.white,
        };
        let foreground = match palette.theme_hue {
            ThemeHue::Dark => palette.white,
            ThemeHue::Light => palette.black,
        };
        let alternate_background_color = if different_color_alternates {
            match palette.theme_hue {
                ThemeHue::Dark => palette.white,
                ThemeHue::Light => palette.black,
            }
        } else {
            palette.fg
        };

        match palette.source {
            PaletteSource::Default => ColoredElements {
                selected: SegmentStyle {
                    prefix_separator: style!(background, palette.green),
                    char_left_separator: style!(background, palette.green).bold(),
                    char_shortcut: style!(palette.red, palette.green).bold(),
                    char_right_separator: style!(background, palette.green).bold(),
                    styled_text: style!(background, palette.green).bold(),
                    suffix_separator: style!(palette.green, background).bold(),
                },
                unselected: SegmentStyle {
                    prefix_separator: style!(background, palette.fg),
                    char_left_separator: style!(background, palette.fg).bold(),
                    char_shortcut: style!(palette.red, palette.fg).bold(),
                    char_right_separator: style!(background, palette.fg).bold(),
                    styled_text: style!(background, palette.fg).bold(),
                    suffix_separator: style!(palette.fg, background),
                },
                unselected_alternate: SegmentStyle {
                    prefix_separator: style!(background, alternate_background_color),
                    char_left_separator: style!(background, alternate_background_color).bold(),
                    char_shortcut: style!(palette.red, alternate_background_color).bold(),
                    char_right_separator: style!(background, alternate_background_color).bold(),
                    styled_text: style!(background, alternate_background_color).bold(),
                    suffix_separator: style!(alternate_background_color, background),
                },
                disabled: SegmentStyle {
                    prefix_separator: style!(background, palette.fg),
                    char_left_separator: style!(background, palette.fg).dimmed().italic(),
                    char_shortcut: style!(background, palette.fg).dimmed().italic(),
                    char_right_separator: style!(background, palette.fg).dimmed().italic(),
                    styled_text: style!(background, palette.fg).dimmed().italic(),
                    suffix_separator: style!(palette.fg, background),
                },
                superkey_prefix: style!(foreground, background).bold(),
                superkey_suffix_separator: style!(background, background),
                modifier: style!(palette.orange, background).bold(),
                key: style!(palette.green, background).bold(),
                text: style!(foreground, background),
            },
            PaletteSource::Xresources => ColoredElements {
                selected: SegmentStyle {
                    prefix_separator: style!(background, palette.green),
                    char_left_separator: style!(palette.fg, palette.green).bold(),
                    char_shortcut: style!(palette.red, palette.green).bold(),
                    char_right_separator: style!(palette.fg, palette.green).bold(),
                    styled_text: style!(background, palette.green).bold(),
                    suffix_separator: style!(palette.green, background).bold(),
                },
                unselected: SegmentStyle {
                    prefix_separator: style!(background, palette.fg),
                    char_left_separator: style!(background, palette.fg).bold(),
                    char_shortcut: style!(palette.red, palette.fg).bold(),
                    char_right_separator: style!(background, palette.fg).bold(),
                    styled_text: style!(background, palette.fg).bold(),
                    suffix_separator: style!(palette.fg, background),
                },
                unselected_alternate: SegmentStyle {
                    prefix_separator: style!(background, alternate_background_color),
                    char_left_separator: style!(background, alternate_background_color).bold(),
                    char_shortcut: style!(palette.red, alternate_background_color).bold(),
                    char_right_separator: style!(background, alternate_background_color).bold(),
                    styled_text: style!(background, alternate_background_color).bold(),
                    suffix_separator: style!(alternate_background_color, background),
                },
                disabled: SegmentStyle {
                    prefix_separator: style!(background, palette.fg),
                    char_left_separator: style!(background, palette.fg).dimmed(),
                    char_shortcut: style!(background, palette.fg).dimmed(),
                    char_right_separator: style!(background, palette.fg).dimmed(),
                    styled_text: style!(background, palette.fg).dimmed(),
                    suffix_separator: style!(palette.fg, background),
                },
                superkey_prefix: style!(background, palette.fg).bold(),
                superkey_suffix_separator: style!(palette.fg, background),
                modifier: style!(palette.orange, background).bold(),
                key: style!(palette.green, background).bold(),
                text: style!(foreground, background),
            },
        }
    }

    pub fn style_key_with_modifier(&self, keyvec: &[Key]) -> Vec<ANSIString<'static>> {
        // Nothing to do, quit...
        if keyvec.is_empty() { return vec![]; }

        let mut ret = vec![];

        let mut keyvec_iter = keyvec.iter();
        let maybe_modifier = match keyvec_iter.next() {
            // Check if all modifiers are the same, if keys exist
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
            self.modifier.paint(modifier_str)
        };
        ret.push(painted_modifier);

        // Prints key group start
        let group_start_str = if no_modifier { "<" } else { " + <" };
        ret.push(self.text.paint(group_start_str));

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

        let key_string = key.join("");
        let key_separator = match &key_string[..] {
            // Special handling of some pre-defined keygroups
            "HJKL" => "",
            "hjkl" => "",
            "←↓↑→" => "",
            "←→" => "",
            "↓↑" => "",
            // Default separator
            _ => "|",
        };

        for (idx, key) in key.iter().enumerate() {
            if idx > 0 && !key_separator.is_empty() {
                ret.push(self.text.paint(key_separator));
            }
            ret.push(self.key.paint(key.clone()));
        }

        // Prints key group end
        let group_end_str = ">";
        ret.push(self.text.paint(group_end_str));

        ret
    }
}