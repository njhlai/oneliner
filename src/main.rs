mod modules;

use zellij_tile::prelude::*;

use crate::modules::status_line::StatusLine;

static ARROW_SEPARATOR: &str = "";
static MORE_MSG: &str = " ... ";

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    mode_info: ModeInfo,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self) {
        set_selectable(false);
        subscribe(&[EventType::ModeUpdate, EventType::TabUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::ModeUpdate(mode_info) => {
                should_render = self.mode_info != mode_info;
                self.mode_info = mode_info;
            }
            Event::TabUpdate(tabs) => {
                should_render = self.tabs != tabs;
                self.tabs = tabs;
            }
            _ => {},
        }

        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let separator = if !self.mode_info.capabilities.arrow_fonts {
            ARROW_SEPARATOR
        } else {
            ""
        };

        let status = StatusLine::build(&self.mode_info, cols, separator);
        let background = match self.mode_info.style.colors.theme_hue {
            ThemeHue::Dark => self.mode_info.style.colors.black,
            ThemeHue::Light => self.mode_info.style.colors.white,
        };

        match background {
            PaletteColor::Rgb((r, g, b)) => {
                print!("{}\u{1b}[48;2;{};{};{}m\u{1b}[0K", status, r, g, b);
            },
            PaletteColor::EightBit(color) => {
                print!("{}\u{1b}[48;5;{}m\u{1b}[0K", status, color);
            },
        }
    }
}