// rpass: a password manager based on pass, written in rust
// Copyright (C) 2020, Tibor Schneider
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see http://www.gnu.org/licenses/

use std::fs;
use std::collections::HashMap;

use uuid::Uuid;
use dirs::home_dir;

use crate::errors::{Error, Result};
use crate::pass::index;
use crate::pass::entry::Entry;
use crate::commands::utils::{confirm, gen_path_interactive, two_options};
use crate::def;
use crate::config::CFG;

pub fn fix_index() -> Result<()> {

    let index_file = index::get_index()?;
    let path_lookup = index::to_hashmap(&index_file);

    let mut uuid_folder = home_dir().unwrap();
    uuid_folder.push(def::ROOT_FOLDER);
    uuid_folder.push(CFG.main.uuid_folder);

    if !uuid_folder.is_dir() {
        return Err(Error::ManagedFolderNotFound);
    }

    for key_file in fs::read_dir(uuid_folder)? {
        let key_file = key_file?;
        let key_path = key_file.path();
        let key_name = key_file.file_name().into_string().unwrap();

        if key_path.is_dir() {
            println!("[Warning] uuids folder should not contain any folders: {}", key_name);
            continue;
        }

        let name_parts: Vec<&str> = key_name.split(".").collect();

        if name_parts.len() != 2 || name_parts[1] != def::ENTRY_EXTENSION {
            println!("[Warning] unrecognized file: {}", key_name);
            continue;
        }

        if key_name == CFG.main.index_file {
            // skip index cile
            continue;
        }

        let entry_id = match Uuid::parse_str(name_parts[0]) {
            Ok(x) => x,
            Err(_) => { println!("[Warning] invalid uuid: {}", key_name);
                        continue; }
        };

        check_fix_entry(entry_id, &path_lookup)?;
        
    }

    Ok(())
}

fn check_fix_entry(entry_id: Uuid, path_lookup: &HashMap<Uuid, &str>) -> Result<()> {

    let mut entry = Entry::get(entry_id)?;

    match path_lookup.get(&entry.uuid) {
        Some(stored_path) => { // uuid is found in index file
            match stored_path == entry.path.as_ref().unwrap() {
                true => {
                    println!("Entry at {} is correct!", stored_path);
                }, // nothing to do
                false => {  // different paths stored
                    println!("\nPath in entry and in index does not match!\n{}", entry);
                    println!("1: Path in index: {}", stored_path);
                    println!("2: Path in entry: {}", entry.path.as_ref().unwrap());
                    print!("Choose path from: ");
                    if two_options("index", "entry") {
                        // use path from index
                        entry.change_path_keep_index(stored_path.to_string())?;
                    } else {
                        // use path from entry
                        index::mv(entry.uuid, entry.path.unwrap())?;
                    }
                }
            }
        },
        None => { // uuid is not found in the index file
            // check if entry has a path stored
            match entry.path.clone() {
                Some(path) => { // generate index entry to the stored path
                    println!("\nEntry is not present in the index!\n{}", entry);
                    if confirm(format!("Create index at {}", path), false) {
                        index::insert(entry.uuid, &path)?;
                    }
                },
                None => { // no path can be found
                    println!("\nEntry is not present in the index and has no path information!\n{}", entry);
                    if confirm("Create index and move entry to new location?", false) {
                        match gen_path_interactive() {
                            Ok(path) => { println!("Move entry to {}", path);
                                          entry.change_path(path)?; },
                            _ => println!("Skipped!")
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
