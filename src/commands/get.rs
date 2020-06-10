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

use crate::errors::Result;
use crate::commands::utils::{choose_entry, copy_to_clipboard};
use crate::commands::edit;
use crate::pass::entry::Entry;
use crate::config::CFG;
use crate::def;

use rofi::{Rofi, Format};

pub fn get(path: Option<&str>,
           id: Option<&str>,
           use_rofi: bool,
           only_password: bool) -> Result<()> {

    let mut entry = choose_entry(path, id, use_rofi)?;
    if use_rofi {
        get_rofi_menu(&mut entry)
    } else if only_password {
        println!("{}", entry.password);
        Ok(())
    } else {
        entry.hidden = false;
        println!("{:?}", entry);
        Ok(())
    }

}

fn get_rofi_menu(entry: &mut Entry) -> Result<()> {
    loop {
        let mut lines: Vec<String> = entry.get_rofi_lines();
        lines.push(String::new());
        if entry.hidden {
            lines.push(def::format_button(def::DISPLAY_BTN_SHOW_PWD));
        } else {
            lines.push(def::format_button(def::DISPLAY_BTN_HIDE_PWD));
        }
        lines.push(def::format_button(def::DISPLAY_BTN_EDIT_ENTRY));
        lines.push(def::format_small(def::DISPLAY_BTN_MAIN_MENU));

        match Rofi::new(&lines)
            .pango()
            .prompt("Entry")
            .return_format(Format::StrippedText)
            .theme(CFG.theme.theme_name)
            .run() {
            Ok(s) => match get_menu_action(s) {
                GetMenuAction::CopyPath     => copy_to_clipboard(entry.path.clone().unwrap(), "path", Some(5000))?,
                GetMenuAction::CopyUuid     => copy_to_clipboard(format!("{}", entry.uuid), "UUID", Some(5000))?,
                GetMenuAction::CopyUsername => copy_to_clipboard(entry.username.clone().unwrap(), "Username", Some(5000))?,
                GetMenuAction::CopyPassword => copy_to_clipboard(entry.password.clone(), "Password", Some(5000))?,
                GetMenuAction::CopyUrl      => copy_to_clipboard(entry.url.clone().unwrap(), "URL", Some(5000))?,
                GetMenuAction::CopyOther(s) => copy_to_clipboard(s, "Custom entry", Some(5000))?,
                GetMenuAction::ShowPassword => entry.hidden = false,
                GetMenuAction::HidePassword => entry.hidden = true,
                GetMenuAction::EditEntry    => { edit(None, Some(format!("{}", entry.uuid).as_str()), true)?;
                                                 break; },
                GetMenuAction::Exit         => break
            }
            Err(_) => break
        }
    }
    Ok(())
}

enum GetMenuAction {
    CopyPath,
    CopyUuid,
    CopyUsername,
    CopyPassword,
    CopyUrl,
    CopyOther(String),
    ShowPassword,
    HidePassword,
    EditEntry,
    Exit
}

fn get_menu_action(s: String) -> GetMenuAction {
    if s == def::DISPLAY_BTN_SHOW_PWD { GetMenuAction::ShowPassword }
    else if s == def::DISPLAY_BTN_HIDE_PWD { GetMenuAction::HidePassword }
    else if s == def::DISPLAY_BTN_EDIT_ENTRY { GetMenuAction::EditEntry }
    else if s.starts_with(def::DISPLAY_PATH) { GetMenuAction::CopyPath }
    else if s.starts_with(def::DISPLAY_UUID) { GetMenuAction::CopyUuid }
    else if s.starts_with(def::DISPLAY_USER) { GetMenuAction::CopyUsername }
    else if s.starts_with(def::DISPLAY_PASS) { GetMenuAction::CopyPassword }
    else if s.starts_with(def::DISPLAY_URL) { GetMenuAction::CopyUrl }
    else if s.len() > 0 && s != def::DISPLAY_BTN_MAIN_MENU {GetMenuAction::CopyOther(s.clone()) }
    else { GetMenuAction::Exit }
}
