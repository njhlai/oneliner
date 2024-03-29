mod modules;

use zellij_tile::prelude::*;

use modules::colored_elements::ColoredElements;
use modules::status_line::StatusLine;

static ARROW_SEPARATOR: &str = "";

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    mode_info: ModeInfo,
}

impl ZellijPlugin for State {
    fn load(&mut self, _: std::collections::BTreeMap<String, String>) {
        set_selectable(true);
        request_permission(&[PermissionType::ReadApplicationState]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::PermissionRequestResult,
        ]);
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
            Event::PermissionRequestResult(_) => {
                should_render = true;
                set_selectable(false);
                unsubscribe(&[EventType::PermissionRequestResult]);
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

register_plugin!(State);
