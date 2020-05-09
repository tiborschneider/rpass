use std::io::prelude::*;                                                           
use std::io;
use std::io::{Error, ErrorKind, Write, BufReader};
use std::{thread, time};
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs::{File, remove_file, OpenOptions};

use uuid::Uuid;
use rpassword;
use fake::{Fake, faker};
use text_io::read;
use clipboard::{ClipboardProvider, ClipboardContext};
use notify_rust::{Notification, NotificationUrgency, Timeout};

use crate::pass;

const DMENU_ARGS: [&str; 10] = ["-l", "10", "-fn", "SauceCodePro Nerd Font-20", "-nb", "#22242c", "-sb", "#30333f", "-sf", "#5294e2"];
const DMENU_ARGS_ACTION: [&str; 8] = ["-fn", "SauceCodePro Nerd Font-20", "-nb", "#22242c", "-sb", "#30333f", "-sf", "#5294e2"];


pub fn delete(path: Option<&str>,
              id: Option<&str>,
              force: bool) {
    let entry: pass::entry::Entry = match (path, id) {
        (Some(path), None) => {
            let index_list = pass::index::get_index().expect("Cannot get index list!");
            let uuid_lookup = pass::index::to_hashmap_reverse(&index_list);
            let entry_id = match uuid_lookup.get(path) {
                Some(id) => id,
                None => panic!("Cannot find an entry at the given path")
            };
            pass::entry::Entry::get(entry_id.clone()).expect("Cannot find the entry based on the UUID in the index file!")
        },
        (None, Some(id)) => {
            let id = Uuid::parse_str(id).expect("UUID is invalid!");
            pass::entry::Entry::get(id).expect("Cannot find the given UUID")
        },
        (None, None)     => select_entry().expect("Cannot get the chosen entry"),
        _                => panic!("This should not happen!")
    };


    if !force {
        println!("{}", entry);
        match confirm("Are you sure to delete this entry?") {
            true  => { pass::index::remove(entry.uuid).expect("Could not remove the entry!"); },
            false => { println!("Action aborted, nothing changed"); }
        }
    }
}

// Private Functions

fn select_entry() -> Result<pass::entry::Entry, Error> {

    let index_list = pass::index::get_index().expect("Cannot get index list!");
    let index_list_clone = index_list.clone();
    let uuid_lookup = pass::index::to_hashmap_reverse(&index_list_clone);

    // rewrite the index list
    let mut p = Command::new("dmenu")
        .args(&DMENU_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut writer) = p.stdin.take() {
        // write all elements
        for (_, path) in index_list {
            writer.write_all(&format!("{}\n", path).into_bytes())?;
        }
    }

    p.wait()?;

    // read the dmenu result
    let mut entry_path: String = String::new();
    if let Some(ref mut stdout) = p.stdout {
        BufReader::new(stdout).read_to_string(&mut entry_path)?;
    }

    // remove all whitespace characters
    entry_path.retain(|c| !c.is_whitespace());

    if entry_path.len() == 0 {
        return Err(Error::new(ErrorKind::Interrupted, "No option chosen!"))
    }

    // get uuid from reverse lookup
    let entry_id = match uuid_lookup.get(entry_path.as_str()) {
        Some(id) => id,
        None => return Err(Error::new(ErrorKind::Other, "Could not find the chosen entry!"))
    };

    pass::entry::Entry::get(entry_id.clone())

}
