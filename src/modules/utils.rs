use zellij_tile::prelude::*;
use zellij_tile::prelude::actions::{Action, SearchDirection, SearchOption};

pub fn filter_get_superkey(key: &Key, vac: &Vec<Action>) -> Option<&'static str> {
    match vac.first() {
        // No actions defined, ignore
        None => None,
        Some(action) => {
            // Ignore certain "default" keybindings that switch back to normal InputMode.
            // These include: ' ', '\n', 'Esc'
            if matches!(key, Key::Char(' ') | Key::Char('\n') | Key::Esc) {
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
                    )
                },
                // `Quit` action
                Action::Quit => true,
                // Not a `SwitchToMode` or `Quit` action, ignore
                _ => false,
            };

            if is_mode_switch_or_quit_action {
                return match key {
                    Key::Ctrl(_) => Some("Ctrl"),
                    Key::Alt(_) => Some("Alt"),
                    _ => None,
                };
            } else { None }
        },
    }
}

pub fn to_key(keybinds: &Vec<(Key, Vec<Action>)>, action: &[Action]) -> Option<Key> {
    keybinds.iter()
        // Get keybinds which match specified action
        .filter_map(|(key, actions)| {
            if actions.as_slice() == action {
                Some(*key)
            } else {
                None
            }
        })
        .into_iter()
        // Filter out certain "default" keybindings: ' ', '\n', 'Esc'
        .filter(|key| !matches!(key, Key::Char(' ') | Key::Char('\n') | Key::Esc))
        // Get only the first Key
        .next()
}


fn action_key(keybinds: &[(Key, Vec<Action>)], action: &[Action]) -> Vec<Key> {
    keybinds.iter()
        // Get keybinds which match specified action
        .filter_map(|(key, actions)| {
            if actions.as_slice() == action {
                Some(*key)
            } else {
                None
            }
        })
        .collect::<Vec<Key>>()
}

fn action_key_group(keymap: &[(Key, Vec<Action>)], actions: &[&[Action]]) -> Vec<Key> {
    let mut ret = vec![];
    for action in actions {
        ret.extend(action_key(keymap, *action));
    }
    ret
}

#[rustfmt::skip]
pub fn get_keys_and_hints(mi: &ModeInfo) -> Vec<(String, String, Vec<Key>)> {
    let mut old_keymap = mi.get_mode_keybinds();
    let s = |string: &str| string.to_string();

    // Find a keybinding to get back to "Normal" input mode. In this case we prefer '\n' over other
    // choices. Do it here before we dedupe the keymap below!
    let to_normal_keys = action_key(&old_keymap, &[Action::SwitchToMode(InputMode::Normal)]);
    let to_normal_key = if to_normal_keys.contains(&Key::Char('\n')) {
        vec![Key::Char('\n')]
    } else {
        // Yield `vec![key]` if `to_normal_keys` has at least one key, or an empty vec otherwise.
        to_normal_keys.into_iter().take(1).collect()
    };

    // Sort and deduplicate the keybindings first. We sort after the `Key`s, and deduplicate by
    // their `Action` vectors. An unstable sort is fine here because if the user maps anything to
    // the same key again, anything will happen...
    old_keymap.sort_unstable_by(|(keya, _), (keyb, _)| keya.partial_cmp(keyb).unwrap());

    let mut known_actions: Vec<Vec<Action>> = vec![];
    let mut km = vec![];
    for (key, acvec) in old_keymap {
        if known_actions.contains(&acvec) {
            // This action is known already
            continue;
        } else {
            known_actions.push(acvec.to_vec());
            km.push((key, acvec));
        }
    }

    match mi.mode {
        InputMode::Pane => {
            vec![
                (s("Move focus"), s("Move"),
                    action_key_group(&km, &[&[Action::MoveFocus(Direction::Left)], &[Action::MoveFocus(Direction::Down)],
                        &[Action::MoveFocus(Direction::Up)], &[Action::MoveFocus(Direction::Right)]])),
                (s("New"), s("New"), action_key(&km, &[Action::NewPane(None, None), Action::SwitchToMode(InputMode::Normal)])),
                (s("Close"), s("Close"), action_key(&km, &[Action::CloseFocus, Action::SwitchToMode(InputMode::Normal)])),
                (s("Rename"), s("Rename"),
                    action_key(&km, &[Action::SwitchToMode(InputMode::RenamePane), Action::PaneNameInput(vec![0])])),
                (s("Split down"), s("Down"), action_key(&km, &[Action::NewPane(Some(Direction::Down), None), Action::SwitchToMode(InputMode::Normal)])),
                (s("Split right"), s("Right"), action_key(&km, &[Action::NewPane(Some(Direction::Right), None), Action::SwitchToMode(InputMode::Normal)])),
                (s("Fullscreen"), s("Fullscreen"), action_key(&km, &[Action::ToggleFocusFullscreen, Action::SwitchToMode(InputMode::Normal)])),
                (s("Frames"), s("Frames"), action_key(&km, &[Action::TogglePaneFrames, Action::SwitchToMode(InputMode::Normal)])),
                (s("Floating toggle"), s("Floating"),
                    action_key(&km, &[Action::ToggleFloatingPanes, Action::SwitchToMode(InputMode::Normal)])),
                (s("Embed pane"), s("Embed"), action_key(&km, &[Action::TogglePaneEmbedOrFloating, Action::SwitchToMode(InputMode::Normal)])),
                (s("Next"), s("Next"), action_key(&km, &[Action::SwitchFocus])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Tab => {
            // With the default bindings, "Move focus" for tabs is tricky: It binds all the arrow keys
            // to moving tabs focus (left/up go left, right/down go right). Since we sort the keys
            // above and then dedpulicate based on the actions, we will end up with LeftArrow for
            // "left" and DownArrow for "right". What we really expect is to see LeftArrow and
            // RightArrow.
            // FIXME: So for lack of a better idea we just check this case manually here.
            let old_keymap = mi.get_mode_keybinds();
            let focus_keys_full: Vec<Key> = action_key_group(&old_keymap,
                &[&[Action::GoToPreviousTab], &[Action::GoToNextTab]]);
            let focus_keys = if focus_keys_full.contains(&Key::Left)
                && focus_keys_full.contains(&Key::Right) {
                vec![Key::Left, Key::Right]
            } else {
                action_key_group(&km, &[&[Action::GoToPreviousTab], &[Action::GoToNextTab]])
            };

            vec![
                (s("Move focus"), s("Move"), focus_keys),
                (s("New"), s("New"), action_key(&km, &[Action::NewTab(None, None), Action::SwitchToMode(InputMode::Normal)])),
                (s("Close"), s("Close"), action_key(&km, &[Action::CloseTab, Action::SwitchToMode(InputMode::Normal)])),
                (s("Rename"), s("Rename"),
                    action_key(&km, &[Action::SwitchToMode(InputMode::RenameTab), Action::TabNameInput(vec![0])])),
                (s("Sync"), s("Sync"), action_key(&km, &[Action::ToggleActiveSyncTab, Action::SwitchToMode(InputMode::Normal)])),
                (s("Toggle"), s("Toggle"), action_key(&km, &[Action::ToggleTab])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Resize => {
            vec![
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
                (s("Increase/Decrease size"), s("Increase/Decrease"),
                    action_key_group(&km, &[
                        &[Action::Resize(Resize::Increase, None)],
                        &[Action::Resize(Resize::Decrease, None)]
                    ])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::Move => {
            vec![
                (s("Move"), s("Move"), action_key_group(&km, &[
                    &[Action::MovePane(Some(Direction::Left))], &[Action::MovePane(Some(Direction::Down))],
                    &[Action::MovePane(Some(Direction::Up))], &[Action::MovePane(Some(Direction::Right))]])),
                (s("Next pane"), s("Next"), action_key(&km, &[Action::MovePane(None)])),
            ]
        },
        InputMode::Scroll => {
            vec![
                (s("Scroll"), s("Scroll"),
                    action_key_group(&km, &[&[Action::ScrollDown], &[Action::ScrollUp]])),
                (s("Scroll page"), s("Scroll"),
                    action_key_group(&km, &[&[Action::PageScrollDown], &[Action::PageScrollUp]])),
                (s("Scroll half page"), s("Scroll"),
                    action_key_group(&km, &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]])),
                (s("Edit scrollback in default editor"), s("Edit"),
                    action_key(&km, &[Action::EditScrollback, Action::SwitchToMode(InputMode::Normal)])),
                (s("Enter search term"), s("Search"),
                    action_key(&km, &[Action::SwitchToMode(InputMode::EnterSearch), Action::SearchInput(vec![0])])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::EnterSearch => {
            vec![
                (s("When done"), s("Done"), action_key(&km, &[Action::SwitchToMode(InputMode::Search)])),
                (s("Cancel"), s("Cancel"),
                    action_key(&km, &[Action::SearchInput(vec![27]), Action::SwitchToMode(InputMode::Scroll)])),
            ]
        },
        InputMode::Search => {
            vec![
                (s("Scroll"), s("Scroll"),
                    action_key_group(&km, &[&[Action::ScrollDown], &[Action::ScrollUp]])),
                (s("Scroll page"), s("Scroll"),
                    action_key_group(&km, &[&[Action::PageScrollDown], &[Action::PageScrollUp]])),
                (s("Scroll half page"), s("Scroll"),
                    action_key_group(&km, &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]])),
                (s("Enter term"), s("Search"),
                    action_key(&km, &[Action::SwitchToMode(InputMode::EnterSearch), Action::SearchInput(vec![0])])),
                (s("Search down"), s("Down"), action_key(&km, &[Action::Search(SearchDirection::Down)])),
                (s("Search up"), s("Up"), action_key(&km, &[Action::Search(SearchDirection::Up)])),
                (s("Case sensitive"), s("Case"),
                    action_key(&km, &[Action::SearchToggleOption(SearchOption::CaseSensitivity)])),
                (s("Wrap"), s("Wrap"),
                    action_key(&km, &[Action::SearchToggleOption(SearchOption::Wrap)])),
                (s("Whole words"), s("Whole"),
                    action_key(&km, &[Action::SearchToggleOption(SearchOption::WholeWord)])),
            ]
        },
        InputMode::Session => {
            vec![
                (s("Detach"), s("Detach"), action_key(&km, &[Action::Detach])),
                (s("Select pane"), s("Select"), to_normal_key),
            ]
        },
        InputMode::RenamePane | InputMode::RenameTab => {
            vec![
                (s("When done"), s("Done"), to_normal_key),
                (s("Select pane"), s("Select"), action_key_group(&km, &[
                    &[Action::MoveFocus(Direction::Left)], &[Action::MoveFocus(Direction::Down)],
                    &[Action::MoveFocus(Direction::Up)], &[Action::MoveFocus(Direction::Right)]])),
            ]
        },
        _ => vec![],
    }
}