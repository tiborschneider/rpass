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

use std::fmt;

use rofi::{Format, Rofi, Width};

use crate::commands::utils;
use crate::config::CFG;
use crate::def;
use crate::errors::{Error, Result};
use crate::pass::entry::Entry;

pub fn interactive() -> Result<()> {
    // choose the entry
    let mut entry = utils::choose_entry(None, None, true)?;

    let lines: Vec<String> = vec![
        def::format_button(def::DISPLAY_BTN_TYPE_USERNAME),
        def::format_button(def::DISPLAY_BTN_TYPE_PASSWORD),
        def::format_button(def::DISPLAY_BTN_TYPE_BOTH),
        def::format_small(def::DISPLAY_BTN_SHOW),
        def::format_small(def::DISPLAY_BTN_EXIT),
    ];

    match Rofi::new(&lines)
        .prompt("What to do?")
        .pango()
        .width(Width::Pixels(CFG.theme.main_screen_width))?
        .theme(CFG.theme.theme_name)
        .return_format(Format::StrippedText)
        .run()
    {
        Ok(s) => action_copy_entry(&mut entry, get_copy_action(s)),
        Err(_) => Err(Error::Other("Rofi exited unsuccessfully".to_string())),
    }
}

enum CopyAction {
    Username,
    Password,
    Both,
    Show,
    Exit,
}

impl fmt::Display for CopyAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CopyAction::Username => write!(f, "Username"),
            CopyAction::Password => write!(f, "Password"),
            CopyAction::Both => write!(f, "Both"),
            CopyAction::Show => write!(f, "Show"),
            CopyAction::Exit => write!(f, "Exit"),
        }
    }
}

fn get_copy_action(s: String) -> CopyAction {
    if s == def::DISPLAY_BTN_TYPE_USERNAME {
        CopyAction::Username
    } else if s == def::DISPLAY_BTN_TYPE_PASSWORD {
        CopyAction::Password
    } else if s == def::DISPLAY_BTN_TYPE_BOTH {
        CopyAction::Both
    } else if s == def::DISPLAY_BTN_SHOW {
        CopyAction::Show
    } else {
        CopyAction::Exit
    }
}

fn action_copy_entry(entry: &mut Entry, action: CopyAction) -> Result<()> {
    match action {
        CopyAction::Username => utils::type_to_x11(entry.username.clone().unwrap_or_default()),
        CopyAction::Password => utils::type_to_x11(entry.password.clone()),
        CopyAction::Both => utils::type_to_x11(format!(
            "{}\t{}",
            entry.username.clone().unwrap(),
            entry.password.clone()
        )),
        CopyAction::Show => super::get::get_rofi_menu(entry),
        CopyAction::Exit => Err(Error::Interrupted),
    }
}
