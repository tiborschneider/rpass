use uuid::Uuid;
use rpassword;
use fake::{Fake, faker};
use rustofi::components::ActionList;
use rustofi::RustofiResult;

use crate::pass;
use crate::pass::entry::Entry;
use crate::commands::utils;

pub fn passwd(path: Option<&str>,
              id: Option<&str>,
              new_passwd: Option<&str>,
              generate: Option<usize>) {

    let passwd = match generate {
        Some(x) => Some(faker::internet::en::Password(x..x+1).fake()),
        None => match new_passwd {
            Some(s) => Some(s.to_string()),
            None => None
        }
    };

    match (path, id) {
        (Some(path), None) => {
            let index_list = pass::index::get_index().expect("Cannot get index list!");
            let uuid_lookup = pass::index::to_hashmap_reverse(&index_list);
            let entry_id = match uuid_lookup.get(path) {
                Some(id) => id,
                None => panic!("Cannot find an entry at the given path")
            };
            let mut entry = Entry::get(entry_id.clone()).expect("Cannot find the entry based on the UUID in the index file!");
            change_passwd(&mut entry, passwd);
        },

        (None, Some(id)) => {
            let id = Uuid::parse_str(id).expect("UUID is invalid!");
            let mut entry = Entry::get(id).expect("Cannot find the given UUID");
            change_passwd(&mut entry, passwd);
        },
        
        (None, None) => {
            let passwd_str: String = passwd.unwrap_or("".to_string());
            let path_list = pass::index::get_path_list().expect("cannot get path list from index file");
            let mut rofi = ActionList::new(passwd_str, path_list, Box::new(entry_callback));
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

fn entry_callback(new_passwd: &String, entry_path: &String) -> RustofiResult {
    let index_list = pass::index::get_index().expect("Cannot get index list!");
    let reverse_lookup = pass::index::to_hashmap_reverse(&index_list);
    let entry_id = match reverse_lookup.get(entry_path.as_str()) {
        Some(id) => id,
        None => return RustofiResult::Error
    };

    let mut entry = Entry::get(entry_id.clone()).expect("Cannot find the entry");

    let passwd_opt = if new_passwd.len() > 0 {
        Some(new_passwd.clone())
    } else {
        None
    };

    change_passwd(&mut entry, passwd_opt);

    RustofiResult::Success
}

fn change_passwd(entry: &mut Entry, new_passwd: Option<String>) {
    println!("Cange password of {}", entry);

    let passwd = match new_passwd {
        Some(x) => x,
        None => {
            let mut passwd: String;
            loop {
                passwd = rpassword::prompt_password_stdout("Enter a password: ").expect("Invalid password entered!");
                let rp = rpassword::prompt_password_stdout("Repeat the password: ").expect("Invalid password entered!");
                if passwd == rp {
                    break;
                } else {
                    println!("The two passwords don't match. try again!");
                }
            }
            passwd
        }
    };

    entry.change_password(if passwd.len() > 0 { Some(passwd) } else { None }).expect("Cannot change password");
    
}
