mod modules;

use zellij_tile::prelude::*;

use crate::modules::colored_elements::ColoredElements;
use crate::modules::status_line::StatusLine;

static ARROW_SEPARATOR: &str = "";

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
            _ => {}
        }

        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let mode_info = &(self.mode_info);
        let simplified_ui = mode_info.capabilities.arrow_fonts;
        let colored_elements = ColoredElements::color_elements(&(mode_info.style.colors), simplified_ui);

        let status = StatusLine::build(
            mode_info,
            &(mode_info.get_mode_keybinds()),
            &colored_elements,
            simplified_ui,
            if simplified_ui { "" } else { ARROW_SEPARATOR },
            cols,
        );
        print!("{status}");
    }
}