use zellij_tile::prelude::actions::{Action, SearchDirection, SearchOption};
use zellij_tile::prelude::*;

pub fn filter_get_superkey(entry: &(Key, Vec<Action>)) -> Option<&'static str> {
    match entry.1.first() {
        // No actions defined, ignore
        None => None,
        Some(action) => {
            let key = entry.0;

            // Ignore certain "default" keybindings that switch back to normal InputMode.
            // These include: ' ', '\n', 'Esc'
            if matches!(key, Key::Char(' ' | '\n') | Key::Esc) {
                return None;
            }

            let is_mode_switch_or_quit_action = match action {
                // `SwitchToMode` action
                Action::SwitchToMode(mode) => {
                    matches!(
                        mode,
                        InputMode::Normal
                            | InputMode::Locked
                            | InputMode::Pane
                            | InputMode::Tab
                            | InputMode::Resize
                            | InputMode::Move
                            | InputMode::Search
                            | InputMode::Scroll
                            | InputMode::Session
                            | InputMode::Tmux
                    )
                }
                // `Quit` action
                Action::Quit => true,
                // Not a `SwitchToMode` or `Quit` action, ignore
                _ => false,
            };

            if is_mode_switch_or_quit_action {
                match key {
                    Key::Ctrl(_) => Some("Ctrl"),
                    Key::Alt(_) => Some("Alt"),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

fn action_key(keybinds: &[(Key, Vec<Action>)], action: &[Action]) -> Vec<Key> {
    keybinds
        .iter()
        // Get keybinds which match specified action
        .filter_map(|(key, actions)| if actions.as_slice() == action { Some(*key) } else { None })
        .collect::<Vec<Key>>()
}

fn action_key_group(keybinds: &[(Key, Vec<Action>)], actions: &[&[Action]]) -> Vec<Key> {
    let mut ret = vec![];

    for &action in actions {
        ret.extend(action_key(keybinds, action));
    }

    ret
}

pub fn to_key(keybinds: &[(Key, Vec<Action>)], action: &[Action]) -> Option<Key> {
    action_key(keybinds, action)
        .into_iter()
        // Get the first Key which is not the "default" keybindings: ' ', '\n', 'Esc'
        .find(|key| !matches!(key, Key::Char(' ') | Key::Char('\n') | Key::Esc))
}

pub fn get_keys_and_hints(mode_info: &ModeInfo) -> Vec<(String, String, Vec<Key>)> {
    let mut old_keymap = mode_info.get_mode_keybinds();
    let s = ToString::to_string;

    // Find a keybinding to get back to "Normal" input mode, before keymap deduplication below.
    // Prefer '\n' over other choices.
    let to_normal_keys = action_key(&old_keymap, &[Action::SwitchToMode(InputMode::Normal)]);
    let to_normal_key = if to_normal_keys.contains(&Key::Char('\n')) {
        vec![Key::Char('\n')]
    } else {
        // Take the first key, if possible
        to_normal_keys.into_iter().take(1).collect()
    };

    // Sort and deduplicate the keybindings first.
    // Sort after the `Key`s, and deduplicate by their `Action` vectors.
    // An unstable sort is fine here because if the user maps anything to the same key again, anything will happen...
    old_keymap.sort_unstable_by(|(key_a, _), (key_b, _)| key_a.partial_cmp(key_b).unwrap());

    let mut known_actions = Vec::<Vec<Action>>::new();
    let mut km = Vec::<(Key, Vec<Action>)>::new();
    for (key, actions) in old_keymap {
        if !known_actions.contains(&actions) {
            known_actions.push(actions.clone());
            km.push((key, actions));
        } else if *actions.as_slice() == [Action::GoToNextTab] && key == Key::Right {
            // Modify known key-action only if it's GoToNextTab action
            // Assumption: If Key::Right is configured for GoToNextTab, assume Key::Left is also configured for GoToPreviousTab
            km.retain(|(_, a)| *a.as_slice() != [Action::GoToNextTab]);
            km.push((key, actions));
        }
    }

    match mode_info.mode {
        InputMode::Locked => vec![(s("-- INTERFACE LOCKED --"), s("INTERFACE LOCKED"), vec![])],
        InputMode::Pane => {
            vec![
                (s("New"), s("New"), action_key(&km, &[
                    Action::NewPane(None, None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Move focus"), s("Move"), action_key_group(&km, &[
                    &[Action::MoveFocus(Direction::Left)], &[Action::MoveFocus(Direction::Down)],
                    &[Action::MoveFocus(Direction::Up)], &[Action::MoveFocus(Direction::Right)]
                ])),
                (s("Close"), s("Close"), action_key(&km, &[
                    Action::CloseFocus, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Rename"), s("Rename"), action_key(&km, &[
                    Action::SwitchToMode(InputMode::RenamePane), Action::PaneNameInput(vec![0])
                ])),
                (s("Split down"), s("Down"), action_key(&km, &[
                    Action::NewPane(Some(Direction::Down), None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Split right"), s("Right"), action_key(&km, &[
                    Action::NewPane(Some(Direction::Right), None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Toggle Fullscreen"), s("Fullscreen"), action_key(&km, &[
                    Action::ToggleFocusFullscreen, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Toggle Frames"), s("Frames"), action_key(&km, &[
                    Action::TogglePaneFrames, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Toggle Floating"), s("Floating"),action_key(&km, &[
                    Action::ToggleFloatingPanes, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Toggle Embed"), s("Embed"), action_key(&km, &[
                    Action::TogglePaneEmbedOrFloating, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Next"), s("Next"), action_key(&km, &[Action::SwitchFocus])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Tab => {
            vec![
                (s("New"), s("New"), action_key(&km, &[
                    Action::NewTab(None, vec![], None, None, None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Change Focus"), s("Move"), action_key_group(&km, &[
                    &[Action::GoToPreviousTab], &[Action::GoToNextTab]
                ])),
                (s("Close"), s("Close"), action_key(&km, &[
                    Action::CloseTab, Action::SwitchToMode(InputMode::Normal)]
                )),
                (s("Rename"), s("Rename"), action_key(&km, &[
                    Action::SwitchToMode(InputMode::RenameTab), Action::TabNameInput(vec![0])
                ])),
                (s("Sync"), s("Sync"), action_key(&km, &[
                    Action::ToggleActiveSyncTab, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Break pane to new tab"), s("Break out"), action_key(&km, &[Action::BreakPane, Action::SwitchToMode(InputMode::Normal)])),
                (s("Break pane left/right"), s("Break"), action_key_group(&km, &[
                    &[Action::BreakPaneLeft, Action::SwitchToMode(InputMode::Normal)],
                    &[Action::BreakPaneRight, Action::SwitchToMode(InputMode::Normal)],
                ])),
                (s("Toggle"), s("Toggle"), action_key(&km, &[Action::ToggleTab])),
                (s("Select tab"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Resize => {
            vec![
                (s("Increase/Decrease size"), s("Increase/Decrease"), action_key_group(&km, &[
                        &[Action::Resize(Resize::Increase, None)],
                        &[Action::Resize(Resize::Decrease, None)]
                ])),
                (s("Increase to"), s("Increase"), action_key_group(&km, &[
                    &[Action::Resize(Resize::Increase, Some(Direction::Left))],
                    &[Action::Resize(Resize::Increase, Some(Direction::Down))],
                    &[Action::Resize(Resize::Increase, Some(Direction::Up))],
                    &[Action::Resize(Resize::Increase, Some(Direction::Right))]
                ])),
                (s("Decrease from"), s("Decrease"), action_key_group(&km, &[
                    &[Action::Resize(Resize::Decrease, Some(Direction::Left))],
                    &[Action::Resize(Resize::Decrease, Some(Direction::Down))],
                    &[Action::Resize(Resize::Decrease, Some(Direction::Up))],
                    &[Action::Resize(Resize::Decrease, Some(Direction::Right))]
                ])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Move => {
            vec![
                (s("Switch Location"), s("Move"), action_key_group(&km, &[
                    &[Action::MovePane(Some(Direction::Left))], &[Action::MovePane(Some(Direction::Down))],
                    &[Action::MovePane(Some(Direction::Up))], &[Action::MovePane(Some(Direction::Right))]
                ])),
                (s("Next pane"), s("Next"), action_key(&km, &[Action::MovePane(None)])),
            ]
        },
        InputMode::Scroll => {
            vec![
                (s("Enter search term"), s("Search"), action_key(&km, &[
                    Action::SwitchToMode(InputMode::EnterSearch), Action::SearchInput(vec![0])
                ])),
                (s("Scroll"), s("Scroll"), action_key_group(&km, &[
                    &[Action::ScrollDown], &[Action::ScrollUp]
                ])),
                (s("Scroll page"), s("Scroll"), action_key_group(&km, &[
                    &[Action::PageScrollDown], &[Action::PageScrollUp]
                ])),
                (s("Scroll half page"), s("Scroll"), action_key_group(&km, &[
                    &[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]
                ])),
                (s("Edit scrollback in default editor"), s("Edit"), action_key(&km, &[
                    Action::EditScrollback, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::EnterSearch => {
            vec![
                (s("When done"), s("Done"), action_key(&km, &[Action::SwitchToMode(InputMode::Search)])),
                (s("Cancel"), s("Cancel"), action_key(&km, &[
                    Action::SearchInput(vec![27]), Action::SwitchToMode(InputMode::Scroll)
                ])),
            ]
        },
        InputMode::Search => {
            vec![
                (s("Enter Search term"), s("Search"), action_key(&km, &[
                    Action::SwitchToMode(InputMode::EnterSearch), Action::SearchInput(vec![0])
                ])),
                (s("Scroll"), s("Scroll"), action_key_group(&km, &[
                    &[Action::ScrollDown], &[Action::ScrollUp]
                ])),
                (s("Scroll page"), s("Scroll"), action_key_group(&km, &[
                    &[Action::PageScrollDown], &[Action::PageScrollUp]
                ])),
                (s("Scroll half page"), s("Scroll"), action_key_group(&km, &[
                    &[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]
                ])),
                (s("Search down"), s("Down"), action_key(&km, &[Action::Search(SearchDirection::Down)])),
                (s("Search up"), s("Up"), action_key(&km, &[Action::Search(SearchDirection::Up)])),
                (s("Case sensitive"), s("Case"), action_key(&km, &[Action::SearchToggleOption(SearchOption::CaseSensitivity)])),
                (s("Wrap"), s("Wrap"), action_key(&km, &[Action::SearchToggleOption(SearchOption::Wrap)])),
                (s("Whole words"), s("Whole"), action_key(&km, &[Action::SearchToggleOption(SearchOption::WholeWord)])),
            ]
        },
        InputMode::Session => {
            vec![
                (s("Detach"), s("Detach"), action_key(&km, &[Action::Detach])),
                (s("Session Manager"), s("Manager"), action_key(&km, &[Action::LaunchOrFocusPlugin(Default::default(), true, true, false), Action::SwitchToMode(InputMode::Normal)])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Tmux => {
            vec![
                (s("Move focus"), s("Move"), action_key_group(&km, &[
                    &[Action::MoveFocus(Direction::Left)], &[Action::MoveFocus(Direction::Down)],
                    &[Action::MoveFocus(Direction::Up)], &[Action::MoveFocus(Direction::Right)]
                ])),
                (s("Split down"), s("Down"), action_key(&km, &[
                    Action::NewPane(Some(Direction::Down), None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Split right"), s("Right"), action_key(&km, &[
                    Action::NewPane(Some(Direction::Right), None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Fullscreen"), s("Fullscreen"), action_key(&km, &[
                    Action::ToggleFocusFullscreen, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("New tab"), s("New"), action_key(&km, &[
                    Action::NewTab(None, vec![], None, None, None), Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Rename tab"), s("Rename"), action_key(&km, &[
                    Action::SwitchToMode(InputMode::RenameTab), Action::TabNameInput(vec![0])
                ])),
                (s("Previous Tab"), s("Previous"), action_key(&km, &[
                    Action::GoToPreviousTab, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Next Tab"), s("Next"), action_key(&km, &[
                    Action::GoToNextTab, Action::SwitchToMode(InputMode::Normal)
                ])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::RenamePane => {
            vec![
                (s("When done"), s("Done"), to_normal_key),
                (s("Select pane"), s("Select"), action_key_group(&km, &[
                    &[Action::MoveFocus(Direction::Left)], &[Action::MoveFocus(Direction::Down)],
                    &[Action::MoveFocus(Direction::Up)], &[Action::MoveFocus(Direction::Right)]
                ])),
            ]
        },
        InputMode::RenameTab => {
            vec![
                (s("When done"), s("Done"), to_normal_key),
                (s("Select tab"), s("Select"), action_key_group(&km, &[
                    &[Action::MoveFocusOrTab(Direction::Left)], &[Action::MoveFocusOrTab(Direction::Right)]
                ])),
            ]
        },
        _ => vec![],
    }
}
