// rpass: a password manager based on pass, written in rust
// Copyright (C) 2020, Tibor Schneider
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see http://www.gnu.org/licenses/

use notify_rust::{Notification, NotificationUrgency, Timeout};
use rofi::{Format, Rofi};

use crate::commands::utils::{choose_entry, confirm, notify_action, notify_error, question_rofi};
use crate::commands::{delete, mv, passwd};
use crate::config::CFG;
use crate::def;
use crate::errors::Result;
use crate::pass::entry::Entry;

pub fn edit(path: Option<&str>, id: Option<&str>, use_rofi: bool) -> Result<()> {
    if use_rofi {
        edit_interactive(path, id)
    } else {
        let mut entry = choose_entry(path, id, use_rofi)?;
        entry.edit()
    }
}

fn edit_interactive(path: Option<&str>, id: Option<&str>) -> Result<()> {
    let mut entry = choose_entry(path, id, true)?;
    let entry_id = entry.uuid;

    loop {
        let mut lines: Vec<String> = entry.get_rofi_lines();
        lines.push(def::format_button(def::DISPLAY_BTN_NEW_RAW));
        lines.push(String::new());
        lines.push(def::format_button(def::DISPLAY_BTN_DELETE));
        lines.push(def::format_button(def::DISPLAY_BTN_MAIN_MENU));
        match Rofi::new(&lines)
            .prompt("Edit Entry")
            .theme(CFG.theme.theme_name)
            .pango()
            .return_format(Format::StrippedText)
            .run()
        {
            Ok(s) => match get_menu_action(s) {
                EditMenuAction::EditPath => {
                    match mv(None, Some(format!("{}", entry_id).as_str()), None, true) {
                        Ok(()) => {
                            entry = Entry::get(entry_id)?;
                            notify_action(format!(
                                "Entry moved to {}",
                                entry.path.as_ref().unwrap()
                            ));
                        }
                        Err(e) => notify_error(e),
                    }
                }
                EditMenuAction::EditUuid => {
                    Notification::new()
                        .summary("UUID cannot be modified!")
                        .urgency(NotificationUrgency::Low)
                        .timeout(Timeout::Milliseconds(5000))
                        .show()?;
                }
                EditMenuAction::EditUsername => {
                    match question_rofi("Username") {
                        Ok(new_user) => {
                            entry.change_username(new_user)?;
                            notify_action("Changed username");
                        }
                        Err(e) => notify_error(e),
                    };
                }
                EditMenuAction::EditPassword => {
                    let random_pw = match confirm("Generate a random password?", true) {
                        true => Some(20),
                        false => None,
                    };
                    match passwd(
                        None,
                        Some(format!("{}", entry_id).as_str()),
                        None,
                        random_pw,
                        true,
                    ) {
                        Ok(()) => {
                            entry = Entry::get(entry_id)?;
                            notify_action("Changed password");
                        }
                        Err(e) => notify_error(e),
                    }
                }
                EditMenuAction::EditUrl => match question_rofi("URL") {
                    Ok(new_url) => entry.change_url(new_url)?,
                    Err(e) => notify_error(e),
                },
                EditMenuAction::EditOther(s) => match question_rofi("Edit Raw line") {
                    Ok(new_line) => match entry.change_raw_line(Some(s), new_line) {
                        Ok(()) => notify_action("Changed raw line"),
                        Err(e) => notify_error(e),
                    },
                    Err(e) => notify_error(e),
                },
                EditMenuAction::AddOther => match question_rofi("Create Raw line") {
                    Ok(new_line) => match entry.change_raw_line(None, new_line) {
                        Ok(()) => notify_action("Added raw line"),
                        Err(e) => notify_error(e),
                    },
                    Err(e) => notify_error(e),
                },
                EditMenuAction::Delete => {
                    match delete(None, Some(format!("{}", entry_id).as_str()), false, true) {
                        Ok(()) => break,
                        Err(e) => notify_error(e),
                    }
                }
                EditMenuAction::DoNothing => {}
                EditMenuAction::Exit => break,
            },
            Err(_) => break,
        }
    }

    Ok(())
}

enum EditMenuAction {
    EditPath,
    EditUuid,
    EditUsername,
    EditPassword,
    EditUrl,
    EditOther(String),
    AddOther,
    Delete,
    DoNothing,
    Exit,
}

fn get_menu_action(s: String) -> EditMenuAction {
    if s.starts_with(def::DISPLAY_PATH) {
        EditMenuAction::EditPath
    } else if s.starts_with(def::DISPLAY_UUID) {
        EditMenuAction::EditUuid
    } else if s.starts_with(def::DISPLAY_USER) {
        EditMenuAction::EditUsername
    } else if s.starts_with(def::DISPLAY_PASS) {
        EditMenuAction::EditPassword
    } else if s.starts_with(def::DISPLAY_URL) {
        EditMenuAction::EditUrl
    } else if s == def::DISPLAY_BTN_NEW_RAW {
        EditMenuAction::AddOther
    } else if s == def::DISPLAY_BTN_DELETE {
        EditMenuAction::Delete
    } else if s == def::DISPLAY_RAW {
        EditMenuAction::DoNothing
    } else if !s.is_empty() && s != def::DISPLAY_BTN_MAIN_MENU {
        EditMenuAction::EditOther(s)
    } else {
        EditMenuAction::Exit
    }
}
