use uuid::Uuid;
use rustofi::components::ItemList;
use rustofi::RustofiResult;

use crate::pass;
use crate::pass::entry::Entry;
use crate::commands::utils;

pub fn get(path: Option<&str>,
           id: Option<&str>) {

    match (path, id) {
        (Some(path), None) => {
            let index_list = pass::index::get_index().expect("Cannot get index list!");
            let uuid_lookup = pass::index::to_hashmap_reverse(&index_list);
            let entry_id = match uuid_lookup.get(path) {
                Some(id) => id,
                None => panic!("Cannot find an entry at the given path")
            };
            let mut entry = Entry::get(entry_id.clone()).expect("Cannot find the entry based on the UUID in the index file!");
            print_entry(&mut entry);
        },

        (None, Some(id)) => {
            let id = Uuid::parse_str(id).expect("UUID is invalid!");
            let mut entry = Entry::get(id).expect("Cannot find the given UUID");
            print_entry(&mut entry);
        },
        
        (None, None) => {
            let path_list = pass::index::get_path_list().expect("cannot get path list from index file");
            let mut rofi = ItemList::new(path_list, Box::new(entry_callback));
            match utils::rofi_display_item(&mut rofi, "Select an entry".to_string(), 10) {
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

fn entry_callback(entry_path: &String) -> RustofiResult {
    let index_list = pass::index::get_index().expect("Cannot get index list!");
    let reverse_lookup = pass::index::to_hashmap_reverse(&index_list);
    let entry_id = match reverse_lookup.get(entry_path.as_str()) {
        Some(id) => id,
        None => return RustofiResult::Error
    };

    let mut entry = Entry::get(entry_id.clone()).expect("Cannot find the entry");
    print_entry(&mut entry);

    RustofiResult::Success
}

fn print_entry(entry: &mut Entry) {
    entry.hidden = false;
    println!("{}", entry);
}
