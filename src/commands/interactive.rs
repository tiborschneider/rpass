use std::io::prelude::*;
use std::io::{Error, ErrorKind, Write, BufReader};
use std::path::Path;
use std::fs::{File, remove_file, OpenOptions};

use uuid::Uuid;

use rustofi::components::{ActionList, ItemList};
use rustofi::window::Dimensions;
use rustofi::RustofiResult;

use crate::pass;
use crate::pass::entry::Entry;
use crate::commands::utils;

const LAST_COMMAND_FILE: &str = "/home/tibor/.cache/rpass_last";
const USERNAME: &str = "Username";
const PASSWORD: &str = "Password";
const BOTH: &str = "Both";

pub fn interactive() -> Result<(), Error> {
    // check if there was a previous access to copy both username and password to clipboard
    let result = match previous_entry()? {
        Some(id) => {
            let entry = pass::entry::Entry::get(id)?;
            action_copy_entry(&entry, &PASSWORD.to_string())
        },
        None => {
            // generate the list view
            let path_list = pass::index::get_path_list()?;
            let mut rofi = ItemList::new(path_list, Box::new(choose_action_callback));
            rofi.window = rofi.window.dimensions(Dimensions {width: 900, height: 800, lines: 10, columns: 1});
            utils::rofi_display_item(&mut rofi, "Choose an entry".to_string(), 10)
        }
    };

    match result {
        RustofiResult::Success => Ok(()),
        RustofiResult::Blank   => Err(Error::new(ErrorKind::Interrupted, "Blank option chosen")),
        RustofiResult::Error   => Err(Error::new(ErrorKind::Other, "Unexpected rofi error")),
        RustofiResult::Cancel  => Err(Error::new(ErrorKind::Interrupted, "Rofi cancelled")),
        RustofiResult::Exit    => Err(Error::new(ErrorKind::Interrupted, "Exit option chosen")),
        _                      => Err(Error::new(ErrorKind::Other, "Unexpected rofi error"))
    }
}

fn choose_action_callback(entry_path: &String) -> RustofiResult {
    let index_list = pass::index::get_index().expect("Cannot get index list!");
    let reverse_lookup = pass::index::to_hashmap_reverse(&index_list);
    let entry_id = match reverse_lookup.get(entry_path.as_str()) {
        Some(id) => id,
        None => return RustofiResult::Error
    };

    let entry = Entry::get(entry_id.clone()).expect("Cannot find the entry");

    match (entry.username.is_some(), entry.password.is_some()) {
        (true, false) => action_copy_entry(&entry, &USERNAME.to_string()),
        (false, true) => action_copy_entry(&entry, &PASSWORD.to_string()),
        (true, true)  => ActionList::new(entry,
                                          vec![USERNAME.to_string(),
                                               PASSWORD.to_string(),
                                               BOTH.to_string()],
                                          Box::new(action_copy_entry)).display("Copy".to_string()),
        _             => panic!("No username or password found for entry!")
    }
}

fn action_copy_entry(entry: &Entry, action: &String) -> RustofiResult {
    let entry = entry.clone();
    let mut copy_both: bool = false;
    let entry_to_copy: String = match action.as_str() {
        USERNAME => entry.username.unwrap(),
        PASSWORD => entry.password.unwrap(),
        BOTH => {
            copy_both = true;
            entry.username.unwrap()
        }
        _ => panic!("Unknown action chosen!")
    };

    if copy_both {
        write_next_entry(entry.uuid).expect("Cannot write next entry");
    }

    utils::copy_to_clipboard(entry_to_copy, action, Some(5000)).expect("Cannot copy to clipboard");

    RustofiResult::Success
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
