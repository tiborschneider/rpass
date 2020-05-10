use std::io::Error;

use rustofi::window::{Window, Dimensions};
use notify_rust::{Notification, NotificationUrgency, Timeout};

use crate::commands::utils::{choose_entry, question_rofi, notify_error, confirm, notify_action};
use crate::commands::{mv, delete, passwd};
use crate::pass::entry;
use crate::pass::entry::Entry;

const NEW_LINE_NAME: &str = "<span size='small' fgcolor='#7EAFE9'>New raw line</span>";
const DELETE_OPTION_NAME: &str = "<span size='small' fgcolor='#7EAFE9'>Delete</span>";
const MAIN_MENU_NAME: &str = "<span size='smaller' alpha='50%'>[Main menu]</span>";

pub fn edit(path: Option<&str>,
            id: Option<&str>,
            use_rofi: bool) -> Result<(), Error> {

    if use_rofi {
        edit_interactive(path, id)
    } else {
        let mut entry = choose_entry(path, id)?;
        entry.edit()
    }

}

fn edit_interactive(path: Option<&str>, id: Option<&str>,) -> Result<(), Error> {

    let mut entry = choose_entry(path, id)?;
    let entry_id = entry.uuid.clone();

    loop {
        let mut lines: Vec<String> = entry.get_string().lines().map(|x| x.to_string()).collect();
        lines.push(NEW_LINE_NAME.to_string());
        lines.push(String::new());
        lines.push(DELETE_OPTION_NAME.to_string());
        lines.push(MAIN_MENU_NAME.to_string());
        match Window::new("Edit Entry")
            .dimensions(Dimensions{width: 1000, height: 1000, lines: 3, columns: 1})
            .lines(lines.len() as i32)
            .format('s')
            .add_args(vec!("-i".to_string(), "-markup-rows".to_string()))
            .show(lines.clone()) {
            Ok(s) => {
                match get_menu_action(s) {
                    EditMenuAction::EditPath     => {
                        match mv(None, Some(format!("{}", entry_id).as_str()), None, true) {
                            Ok(()) => {
                                entry = Entry::get(entry_id)?;
                                notify_action(format!("Entry moved to {}", entry.path.as_ref().unwrap()));
                            },
                            Err(e) => notify_error(e)
                        }
                    },
                    EditMenuAction::EditUuid     => {
                        Notification::new()
                            .summary("UUID cannot be modified!")
                            .urgency(NotificationUrgency::Low)
                            .timeout(Timeout::Milliseconds(5000))
                            .show().unwrap();
                    },
                    EditMenuAction::EditUsername => {
                        match question_rofi("Username", entry.username.as_ref()) {
                            Ok(new_user) => {
                                entry.change_username(new_user)?;
                                notify_action("Changed username");
                            },
                            Err(e) => notify_error(e)
                        };
                    },
                    EditMenuAction::EditPassword => {
                        let random_pw = match confirm("Generate a random password?", true) {
                            true => Some(20),
                            false => None
                        };
                        match passwd(None, Some(format!("{}", entry_id).as_str()), None, random_pw, true) {
                            Ok(()) => {
                                entry = Entry::get(entry_id)?;
                                notify_action("Changed password");
                            },
                            Err(e) => notify_error(e)
                        }
                    },
                    EditMenuAction::EditUrl      => {
                        match question_rofi("URL", entry.url.as_ref()) {
                            Ok(new_url) => entry.change_url(new_url)?,
                            Err(e) => notify_error(e)
                        }
                    },
                    EditMenuAction::EditOther(s) => {
                        match question_rofi("Edit Raw line", Some(&s)) {
                            Ok(new_line) => {
                                match entry.change_raw_line(Some(s), new_line) {
                                    Ok(()) => notify_action("Changed raw line"),
                                    Err(e) => notify_error(e)
                                }
                            },
                            Err(e) => notify_error(e)
                        }
                    },
                    EditMenuAction::AddOther => {
                        match question_rofi("Create Raw line", None) {
                            Ok(new_line) => {
                                match entry.change_raw_line(None, new_line) {
                                    Ok(()) => notify_action("Added raw line"),
                                    Err(e) => notify_error(e)
                                }
                            },
                            Err(e) => notify_error(e)
                        }
                    }
                    EditMenuAction::Delete       => {
                        match delete(None, Some(format!("{}", entry_id).as_str()), false, true) {
                            Ok(()) => break,
                            Err(e) => notify_error(e)
                        }
                    }
                    EditMenuAction::DoNothing    => {}
                    EditMenuAction::Exit         => break
                }
            }
            Err(_) => break
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
    if s.starts_with(entry::PANGO_PATH_NAME) { EditMenuAction::EditPath }
    else if s.starts_with(entry::PANGO_UUID_NAME) { EditMenuAction::EditUuid }
    else if s.starts_with(entry::PANGO_USERNAME_NAME) { EditMenuAction::EditUsername }
    else if s.starts_with(entry::PANGO_PASSWORD_NAME) { EditMenuAction::EditPassword }
    else if s.starts_with(entry::PANGO_URL_NAME) { EditMenuAction::EditUrl }
    else if s == NEW_LINE_NAME { EditMenuAction::AddOther }
    else if s == DELETE_OPTION_NAME { EditMenuAction::Delete }
    else if s == entry::PANGO_RAW_NAME { EditMenuAction::DoNothing }
    else if s.len() > 0 && s != MAIN_MENU_NAME { EditMenuAction::EditOther(s.clone()) }
    else { EditMenuAction::Exit }
}

