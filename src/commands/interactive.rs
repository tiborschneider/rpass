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
use std::fs::{remove_file, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use dirs::home_dir;
use rofi::{Format, Rofi, Width};
use uuid::Uuid;

use crate::commands::utils;
use crate::config::CFG;
use crate::def;
use crate::errors::{Error, Result};
use crate::pass::entry::Entry;

pub fn interactive() -> Result<()> {
    // choose the entry
    match previous_entry()? {
        Some(id) => {
            let entry = Entry::get(id)?;
            action_copy_entry(&entry, CopyAction::Password(false))
        }
        None => {
            let entry = utils::choose_entry(None, None, true)?;

            let lines: Vec<String> = vec![
                def::format_button(def::DISPLAY_BTN_CPY_USERNAME_TYPE),
                def::format_button(def::DISPLAY_BTN_CPY_PASSWORD_TYPE),
                def::format_button(def::DISPLAY_BTN_CPY_BOTH_TYPE),
                def::format_button(def::DISPLAY_BTN_CPY_USERNAME),
                def::format_button(def::DISPLAY_BTN_CPY_PASSWORD),
                def::format_button(def::DISPLAY_BTN_CPY_BOTH),
                def::format_small(def::DISPLAY_BTN_EXIT),
            ];

            match Rofi::new(&lines)
                .prompt("What to copy?")
                .pango()
                .width(Width::Pixels(CFG.theme.main_screen_width))?
                .theme(CFG.theme.theme_name)
                .return_format(Format::StrippedText)
                .run()
            {
                Ok(s) => action_copy_entry(&entry, get_copy_action(s)),
                Err(_) => Err(Error::Other("Rofi exited unsuccessfully".to_string())),
            }
        }
    }
}

enum CopyAction {
    Username(bool),
    Password(bool),
    Both(bool),
    Exit,
}

impl fmt::Display for CopyAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CopyAction::Username(false) => write!(f, "Username (clipboard)"),
            CopyAction::Username(true) => write!(f, "Username (type)"),
            CopyAction::Password(false) => write!(f, "Password (clipboard)"),
            CopyAction::Password(true) => write!(f, "Password (type)"),
            CopyAction::Both(false) => write!(f, "Both (clipboard)"),
            CopyAction::Both(true) => write!(f, "Both (type)"),
            CopyAction::Exit => write!(f, "Exit"),
        }
    }
}

fn get_copy_action(s: String) -> CopyAction {
    if s == def::DISPLAY_BTN_CPY_USERNAME {
        CopyAction::Username(false)
    } else if s == def::DISPLAY_BTN_CPY_USERNAME_TYPE {
        CopyAction::Username(true)
    } else if s == def::DISPLAY_BTN_CPY_PASSWORD {
        CopyAction::Password(false)
    } else if s == def::DISPLAY_BTN_CPY_PASSWORD_TYPE {
        CopyAction::Password(true)
    } else if s == def::DISPLAY_BTN_CPY_BOTH {
        CopyAction::Both(false)
    } else if s == def::DISPLAY_BTN_CPY_BOTH_TYPE {
        CopyAction::Both(true)
    } else {
        CopyAction::Exit
    }
}

fn action_copy_entry(entry: &Entry, action: CopyAction) -> Result<()> {
    let mut type_text: bool = false;
    let mut copy_both: bool = false;
    let entry_to_copy: String = match action {
        CopyAction::Username(c) => {
            type_text = c;
            entry.username.clone().unwrap()
        }
        CopyAction::Password(c) => {
            type_text = c;
            entry.password.clone()
        }
        CopyAction::Both(false) => {
            copy_both = true;
            entry.username.clone().unwrap()
        }
        CopyAction::Both(true) => {
            copy_both = false;
            type_text = true;
            format!(
                "{}\t{}",
                entry.username.clone().unwrap(),
                entry.password.clone()
            )
        }
        CopyAction::Exit => return Err(Error::Interrupted),
    };

    let action_str = if copy_both {
        write_next_entry(entry.uuid)?;
        format!("{}", CopyAction::Username(false))
    } else {
        format!("{}", action)
    };

    if type_text {
        utils::type_to_x11(entry_to_copy)
    } else {
        utils::copy_to_clipboard(entry_to_copy, action_str, Some(5000))
    }
}

fn previous_entry() -> Result<Option<Uuid>> {
    let last_command_file = get_last_command_file();
    match last_command_file.exists() {
        false => Ok(None),
        true => {
            let file = File::open(last_command_file.as_path())?;
            let mut buf_reader = BufReader::new(file);
            let mut content = String::new();
            buf_reader.read_line(&mut content)?;
            content.retain(|c| !c.is_whitespace());
            let result = Uuid::parse_str(&content)?;
            remove_file(last_command_file.as_path())?;
            Ok(Some(result))
        }
    }
}

fn write_next_entry(id: Uuid) -> Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(get_last_command_file())?
        .write_all(format!("{}\n", id).as_bytes())?;
    Ok(())
}

fn get_last_command_file() -> PathBuf {
    let mut last_command_file = home_dir().unwrap();
    last_command_file.push(CFG.main.last_command_file);
    last_command_file
}
