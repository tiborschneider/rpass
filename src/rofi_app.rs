use std::io::{Error, ErrorKind};

use notify_rust::{Notification,
                  NotificationUrgency,
                  Timeout};
use rustofi:: window::Window;

use crate::commands::{insert, get, edit};
use crate::commands::utils::{confirm, notify_error};

const GET_NAME: &str = "<span weight='bold' fgcolor='#7EAFE9'>Get Entry</span>";
const NEW_NAME: &str = "<span weight='bold' fgcolor='#7EAFE9'>New Entry</span>";
const EDIT_NAME: &str = "<span weight='bold' fgcolor='#7EAFE9'>Edit Entry</span>";
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
    match Window::new("RPASS - Main Menu").lines(options.len() as i32)
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
        Ok(()) => {
            Notification::new()
                .summary("Action Successful!")
                .urgency(NotificationUrgency::Low)
                .timeout(Timeout::Milliseconds(1000))
                .show().unwrap();
        }
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
