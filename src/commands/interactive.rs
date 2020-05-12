use std::fmt;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Write, BufReader};
use std::path::PathBuf;
use std::fs::{File, remove_file, OpenOptions};

use uuid::Uuid;
use rustofi::window::{Dimensions, Window};
use dirs::home_dir;

use crate::pass::entry::Entry;
use crate::commands::utils;
use crate::def;

pub fn interactive() -> Result<(), Error> {
    // choose the entry
    match previous_entry()? {
        Some(id) => {
            let entry = Entry::get(id)?;
            action_copy_entry(&entry, CopyAction::Password)
        },
        None => {
            let entry = utils::choose_entry(None, None)?;

            let lines: Vec<String> = vec![def::PANGO_COPY_USERNAME_NAME.to_string(),
                                          def::PANGO_COPY_PASSWORD_NAME.to_string(),
                                          def::PANGO_COPY_BOTH_NAME.to_string(),
                                          def::PANGO_EXIT_NAME.to_string()];
            match Window::new("What to copy?")
                .dimensions(Dimensions{width: 400, height: 1000, lines: 4, columns: 1})
                .lines(lines.len() as i32)
                .format('s')
                .add_args(vec!("-i".to_string(), "-markup-rows".to_string()))
                .show(lines.clone()) {
                Ok(s)  => action_copy_entry(&entry, get_copy_action(s)),
                Err(_) => Err(Error::new(ErrorKind::Other, "Rofi exited unsuccessfully"))
            }
        }
    }
}

enum CopyAction {
    Username,
    Password,
    Both,
    Exit
}

impl fmt::Display for CopyAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CopyAction::Username => write!(f, "Username"),
            CopyAction::Password => write!(f, "Password"),
            CopyAction::Both => write!(f, "Both"),
            CopyAction::Exit => write!(f, "Exit"),
        }
    }
}

fn get_copy_action(s: String) -> CopyAction {
    if s == def::PANGO_COPY_USERNAME_NAME { CopyAction::Username }
    else if s == def::PANGO_COPY_PASSWORD_NAME { CopyAction::Password }
    else if s == def::PANGO_COPY_BOTH_NAME { CopyAction::Both }
    else { CopyAction::Exit }
}

fn action_copy_entry(entry: &Entry, action: CopyAction) -> Result<(), Error> {
    let mut copy_both: bool = false;
    let entry_to_copy: String = match action {
        CopyAction::Username => entry.username.clone().unwrap(),
        CopyAction::Password => entry.password.clone(),
        CopyAction::Both => {
            copy_both = true;
            entry.username.clone().unwrap()
        }
        CopyAction::Exit => return Err(Error::new(ErrorKind::Interrupted, "Interrupted"))
    };

    let action_str = if copy_both {
        write_next_entry(entry.uuid)?;
        format!("{}", CopyAction::Username)
    } else {
        format!("{}", action)
    };

    utils::copy_to_clipboard(entry_to_copy, action_str, Some(5000))
}

fn previous_entry() -> Result<Option<Uuid>, Error> {
    let last_command_file = get_last_command_file();
    match last_command_file.exists() {
        false => Ok(None),
        true => {
            let file = File::open(last_command_file.as_path())?;
            let mut buf_reader = BufReader::new(file);
            let mut content = String::new();
            buf_reader.read_line(&mut content)?;
            content.retain(|c| !c.is_whitespace());
            let result = match Uuid::parse_str(&content) {
                Ok(id) => Ok(Some(id)),
                Err(_) => Err(Error::new(ErrorKind::InvalidData, "Cannot parse UUID"))
            };
            remove_file(last_command_file.as_path())?;
            result
        }
    }
}

fn write_next_entry(id: Uuid) -> Result<(), Error> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(get_last_command_file())?
        .write_all(format!("{}\n", id).as_bytes())?;
    Ok(())
}

fn get_last_command_file() -> PathBuf {
    let mut last_command_file = home_dir().unwrap();
    last_command_file.push(def::LAST_COMMAND_FILE);
    last_command_file
}
