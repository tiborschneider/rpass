use std::fmt;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Write, BufReader};
use std::path::Path;
use std::fs::{File, remove_file, OpenOptions};

use uuid::Uuid;

use rustofi::window::{Dimensions, Window};

use crate::pass::entry::Entry;
use crate::commands::utils;

const LAST_COMMAND_FILE: &str = "/home/tibor/.cache/rpass_last";
const USERNAME: &str = "<span fgcolor='#7EAFE9'>Username</span>";
const PASSWORD: &str = "<span fgcolor='#7EAFE9'>Password</span>";
const BOTH: &str = "<span fgcolor='#7EAFE9'>Both</span>";
const EXIT: &str = "<span size='small' alpha='50%'>exit</span>";

pub fn interactive() -> Result<(), Error> {
    // choose the entry
    match previous_entry()? {
        Some(id) => {
            let entry = Entry::get(id)?;
            action_copy_entry(&entry, CopyAction::Password)
        },
        None => {
            let entry = utils::choose_entry(None, None)?;

            let lines: Vec<String> = vec![USERNAME.to_string(),
                                        PASSWORD.to_string(),
                                        BOTH.to_string(),
                                        EXIT.to_string()];
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
    if s == USERNAME { CopyAction::Username }
    else if s == PASSWORD { CopyAction::Password }
    else if s == BOTH { CopyAction::Both }
    else { CopyAction::Exit }
}

fn action_copy_entry(entry: &Entry, action: CopyAction) -> Result<(), Error> {
    let mut copy_both: bool = false;
    let entry_to_copy: String = match action {
        CopyAction::Username => entry.username.clone().unwrap(),
        CopyAction::Password => entry.password.clone().unwrap(),
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
    match Path::new(LAST_COMMAND_FILE).exists() {
        false => Ok(None),
        true => {
            let file = File::open(LAST_COMMAND_FILE)?;
            let mut buf_reader = BufReader::new(file);
            let mut content = String::new();
            buf_reader.read_line(&mut content)?;
            content.retain(|c| !c.is_whitespace());
            let result = match Uuid::parse_str(&content) {
                Ok(id) => Ok(Some(id)),
                Err(_) => Err(Error::new(ErrorKind::InvalidData, "Cannot parse UUID"))
            };
            remove_file(LAST_COMMAND_FILE)?;
            result
        }
    }
}

fn write_next_entry(id: Uuid) -> Result<(), Error> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(LAST_COMMAND_FILE)?
        .write_all(format!("{}\n", id).as_bytes())?;
    Ok(())
}
