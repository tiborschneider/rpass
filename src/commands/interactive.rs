use std::io::prelude::*;                                                           
use std::io::{Error, ErrorKind, Write, BufReader};
use std::{thread, time};
use std::path::Path;
use std::fs::{File, remove_file, OpenOptions};

use uuid::Uuid;
use clipboard::{ClipboardProvider, ClipboardContext};
use notify_rust::{Notification, NotificationUrgency, Timeout};

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

pub fn interactive() {
    // check if there was a previous access to copy both username and password to clipboard
    let result = match previous_entry().expect("Cannot read last command file!") {
        Some(id) => {
            let entry = pass::entry::Entry::get(id).expect("Cannot find UUID!");
            action_copy_entry(&entry, &PASSWORD.to_string())
        },
        None => {
            // generate the list view
            let path_list = pass::index::get_path_list().expect("cannot get path list from index file");
            let mut rofi = ItemList::new(path_list, Box::new(choose_action_callback));
            rofi.window = rofi.window.dimensions(Dimensions {width: 800, height: 800, lines: 10, columns: 1});
            utils::rofi_display_item(&mut rofi, "Choose an entry".to_string(), 10)
        }
    };

    match result {
        RustofiResult::Success => {},
        RustofiResult::Blank   => { println!("Nothing was selected!"); },
        RustofiResult::Error   => { panic!("Something went wrong selecting an entry"); },
        RustofiResult::Cancel  => { println!("Action cancelled!"); },
        RustofiResult::Exit    => { println!("Exit action is selected!"); },
        _                      => { panic!("Unknown result!") }
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

    copy_to_clipboard(entry_to_copy, action).expect("Cannot copy to clipboard");

    if copy_both {
        write_next_entry(entry.uuid).expect("Cannot write next entry");
        delayed_clipboard_clear(false).expect("Cannot clear clipboard");
    } else {
        delayed_clipboard_clear(true).expect("Cannot clear clipboard");
    }
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

fn copy_to_clipboard(s: String, action: &String) -> Result<(), Error> {

    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(ctx) => ctx,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot generate clipboard context"))
    };

    match ctx.set_contents(s) {
        Ok(_) => {},
        Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot set clipboard content!"))
    };

    let action_string = format!("Copied {}", action);

    Notification::new()
        .summary(action_string.as_str())
        .urgency(NotificationUrgency::Normal)
        .timeout(Timeout::Milliseconds(5000))
        .show().unwrap();

    Ok(())
}

fn delayed_clipboard_clear(do_clear: bool) -> Result<(), Error> {

    // wait for 5 seconds
    let ten_millis = time::Duration::from_millis(5000);
    thread::sleep(ten_millis);

    if do_clear {
        // clear the clipboard
        let mut ctx: ClipboardContext = match ClipboardProvider::new() {
            Ok(ctx) => ctx,
            Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot generate clipboard context"))
        };
        match ctx.set_contents(" ".to_string()) {
            Ok(_) => {},
            Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot set clipboard content!"))
        };

        Notification::new()
            .summary("Clipboard cleared!")
            .urgency(NotificationUrgency::Low)
            .timeout(Timeout::Milliseconds(1000))
            .show().unwrap();
    }

    Ok(())
}
