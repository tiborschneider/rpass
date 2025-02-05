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

use rofi::{Format, Rofi, Width};
use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};
use uuid::Uuid;

use crate::commands::utils;
use crate::config::CFG;
use crate::def;
use crate::errors::{Error, Result};
use crate::pass::entry::Entry;

#[derive(Debug, Serialize, Deserialize)]
struct LastAction {
    timestamp: SystemTime,
    uuid: Uuid,
    action: CopyAction,
}

const LAST_ACTION_TIMEOUT_SECS: u64 = 60;
const LAST_ACTION_FILENAME: &str = ".rpass_last_action.json";

pub fn interactive() -> Result<()> {
    // first, check if we have an interactive file.
    if let Some(last_action) = get_last_action() {
        // check if the action was called within one minute
        if last_action
            .timestamp
            .elapsed()
            .map(|x| x.as_secs())
            .unwrap_or(LAST_ACTION_TIMEOUT_SECS)
            < LAST_ACTION_TIMEOUT_SECS
        {
            // apply the last action.
            let mut entry = Entry::get(last_action.uuid)?;
            action_copy_entry(&mut entry, last_action.action)?;
            return Ok(());
        }
    }

    // choose the entry
    let mut entry = utils::choose_entry(None, None, true)?;

    let lines: Vec<String> = vec![
        def::format_button(def::DISPLAY_BTN_TYPE_BOTH),
        def::format_button(def::DISPLAY_BTN_TYPE_ONE_AT_A_TIME),
        def::format_button(def::DISPLAY_BTN_TYPE_PASSWORD),
        def::format_button(def::DISPLAY_BTN_TYPE_USERNAME),
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

#[derive(Debug, Serialize, Deserialize)]
enum CopyAction {
    Both,
    OneAtATime,
    Password,
    Username,
    Show,
    Exit,
}

impl fmt::Display for CopyAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CopyAction::Both => write!(f, "Both"),
            CopyAction::OneAtATime => write!(f, "Username, then Password"),
            CopyAction::Password => write!(f, "Only Password"),
            CopyAction::Username => write!(f, "Only Username"),
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
    } else if s == def::DISPLAY_BTN_TYPE_ONE_AT_A_TIME {
        CopyAction::OneAtATime
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
        CopyAction::OneAtATime => {
            // first, enter the username
            utils::type_to_x11(entry.username.clone().unwrap_or_default())?;
            // then, write the uuid to the temporary file, to remember that we want to enter the
            // password of that uuid next
            write_last_action(LastAction {
                timestamp: SystemTime::now(),
                uuid: entry.uuid,
                action: CopyAction::Password,
            })
        }
        CopyAction::Show => super::get::get_rofi_menu(entry),
        CopyAction::Exit => Err(Error::Interrupted),
    }
}

fn get_last_action() -> Option<LastAction> {
    let mut file = std::env::temp_dir();
    file.push(LAST_ACTION_FILENAME);

    // read the content. If we cannot read it, return None
    let content = std::fs::read_to_string(&file).ok()?;

    // delete the file, ignoring any errors that may arise.
    let _ = std::fs::remove_file(file);

    serde_json::from_str(&content).ok()
}

fn write_last_action(action: LastAction) -> Result<()> {
    let mut file = std::env::temp_dir();
    file.push(LAST_ACTION_FILENAME);

    // generate the content string
    let content = serde_json::to_string_pretty(&action).unwrap();
    std::fs::write(file, content)?;
    Ok(())
}
