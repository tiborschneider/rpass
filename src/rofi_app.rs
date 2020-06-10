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
use crate::def;
use crate::commands::{insert, get, edit};
use crate::commands::utils::{confirm, notify_error};

use rofi::{Rofi, Format, Width};

#[derive(Debug)]
enum Action {
    Get,
    New,
    Edit,
    Exit
}

pub fn rofi_app() -> Result<()> {
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
    let options = vec![def::format_big_button(def::DISPLAY_BTN_MM_GET),
                       def::format_big_button(def::DISPLAY_BTN_MM_NEW),
                       def::format_big_button(def::DISPLAY_BTN_MM_EDIT),
                       def::format_small(def::DISPLAY_BTN_MM_EXIT)];
    match Rofi::new(&options)
        .prompt("RPASS - Main Menu")
        .pango()
        .width(Width::Pixels(350)).unwrap()
        .return_format(Format::StrippedText)
        .run(){
        Ok(s) => {
            println!("{}", s);
            match s.as_str() {
                def::DISPLAY_BTN_MM_GET => Action::Get,
                def::DISPLAY_BTN_MM_NEW => Action::New,
                def::DISPLAY_BTN_MM_EDIT => Action::Edit,
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
        Action::Exit => Ok(()),
    } {
        Ok(()) => {}
        Err(e) => notify_error(e)
    }
}

fn action_new() -> Result<()> {
    let random_pw = match confirm("Generate a random password?", true) {
        true => Some(20),
        false => None
    };
    insert(None, None, None, None, random_pw, true)
}

fn action_get() -> Result<()> {
    get(None, None, true, false)
}

fn action_edit() -> Result<()> {
    edit(None, None, true)
}
