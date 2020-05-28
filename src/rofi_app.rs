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

use std::io::{Error, ErrorKind};

use rustofi::window::{Window, Dimensions};

use crate::commands::{insert, get, edit};
use crate::commands::utils::{confirm, notify_error};

const GET_NAME: &str = "<span fgcolor='#7EAFE9'>Get Entry</span>";
const NEW_NAME: &str = "<span fgcolor='#7EAFE9'>New Entry</span>";
const EDIT_NAME: &str = "<span fgcolor='#7EAFE9'>Edit Entry</span>";
const EXIT_NAME: &str = "<span size='small' fgcolor='#FFFFFF80'>Exit</span>";

#[derive(Debug)]
enum Action {
    Get,
    New,
    Edit,
    Exit
}

pub fn rofi_app() -> Result<(), Error> {
    // endless loop
    loop {
        match main_menu() {
            Action::Exit => break,
            action => action_wrapper(action)
        }
    }

    Ok(())

}

fn main_menu() -> Action {
    let options = vec![GET_NAME.to_string(),
                       NEW_NAME.to_string(),
                       EDIT_NAME.to_string(),
                       EXIT_NAME.to_string()];
    match Window::new("RPASS - Main Menu")
        .dimensions(Dimensions{width: 400, height: 400, lines: 1, columns: 1})
        .lines(options.len() as i32)
        .add_args(vec!("-i".to_string(), "-markup-rows".to_string()))
        .format('s')
        .show(options) {
        Ok(s) => {
            match s.as_ref() {
                GET_NAME => Action::Get,
                NEW_NAME => Action::New,
                EDIT_NAME => Action::Edit,
                _ => Action::Exit,
            }
        },
        Err(_) => Action::Exit
    }
}

fn action_wrapper(action: Action) {
    match match action {
        Action::Get => action_get(),
        Action::New => action_new(),
        Action::Edit => action_edit(),
        _ => Err(Error::new(ErrorKind::Other, "Not Implemented"))
    } {
        Ok(()) => {}
        Err(e) => notify_error(e)
    }
}

fn action_new() -> Result<(), Error> {
    let random_pw = match confirm("Generate a random password?", true) {
        true => Some(20),
        false => None
    };
    insert(None, None, None, None, random_pw, true)
}

fn action_get() -> Result<(), Error> {
    get(None, None, true)
}

fn action_edit() -> Result<(), Error> {
    edit(None, None, true)
}
