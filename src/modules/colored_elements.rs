use ansi_term::Style;
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
            },
        }
    }
}