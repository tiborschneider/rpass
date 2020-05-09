use std::io;
use std::io::prelude::*;                                                           
use uuid::Uuid;
use text_io::read;
use rustofi::components::ActionList;
use rustofi::window::Dimensions;
use rustofi::RustofiResult;

use crate::pass;
use crate::pass::entry::Entry;
use crate::commands::utils;

pub fn delete(path: Option<&str>,
              id: Option<&str>,
              force: bool) {

    match (path, id) {
        (Some(path), None) => {
            let index_list = pass::index::get_index().expect("Cannot get index list!");
            let uuid_lookup = pass::index::to_hashmap_reverse(&index_list);
            let entry_id = match uuid_lookup.get(path) {
                Some(id) => id,
                None => panic!("Cannot find an entry at the given path")
            };
            delete_entry(entry_id.clone(), force);
        },

        (None, Some(id)) => {
            let id = Uuid::parse_str(id).expect("UUID is invalid!");
            delete_entry(id.clone(), force);
        },
        
        (None, None) => {
            let path_list = pass::index::get_path_list().expect("cannot get path list from index file");
            let mut rofi = ActionList::new(force.clone(), path_list, Box::new(entry_callback));
            rofi.window = rofi.window.dimensions(Dimensions {width: 800, height: 800, lines: 10, columns: 1});
            match utils::rofi_display_action(&mut rofi, "Select an entry".to_string(), 10) {
                RustofiResult::Success => {},
                RustofiResult::Blank   => { println!("Nothing was selected!"); },
                RustofiResult::Error   => { panic!("Something went wrong selecting an entry"); },
                RustofiResult::Cancel  => { println!("Action cancelled!"); },
                RustofiResult::Exit    => { println!("Exit action is selected!"); },
                _                      => { panic!("Unknown result!") }
            }
        },

        _ => panic!("This should not happen")
    }
}

fn entry_callback(force: &bool, entry_path: &String) -> RustofiResult {
    let index_list = pass::index::get_index().expect("Cannot get index list!");
    let reverse_lookup = pass::index::to_hashmap_reverse(&index_list);
    let entry_id = match reverse_lookup.get(entry_path.as_str()) {
        Some(id) => id,
        None => return RustofiResult::Error
    };

    delete_entry(entry_id.clone(), force.clone());

    RustofiResult::Success
}

fn delete_entry(entry_id: Uuid, force: bool) {

    let entry = Entry::get(entry_id).expect("Entry does not exist");
    if !force {
        println!("{}", entry);
        match confirm("Are you sure to delete this entry?") {
            true  => { pass::index::remove(entry.uuid).expect("Could not remove the entry!"); },
            false => { println!("Action aborted, nothing changed"); }
        }
    } else {
        pass::index::remove(entry.uuid).expect("Could not remove the entry!");
    }

}

fn confirm(q: &str) -> bool {
    print!("{} [y/N]: ", q);
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    answer == "y" || answer == "Y"
}
