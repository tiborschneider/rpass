use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::fs;
use std::path::Path;

use dirs::home_dir;
use uuid::Uuid;
use text_io::read;

use crate::pass::index;
use crate::pass::entry::Entry;
use crate::commands::utils;
use crate::def;

pub fn init(force: bool) -> Result<(), Error> {

    let mut root_folder = home_dir().unwrap();
    root_folder.push(def::ROOT_FOLDER);
    let root_folder_len: usize = root_folder.as_path().to_str().unwrap().len() + 1;

    // check if the password store folder exists
    if !root_folder.is_dir() {
        // pass is not yet initialized!
        println!("[Error] Pass is not yet initialized! Initialize it by running \"pass init\"!");
        return Err(Error::new(ErrorKind::NotFound, "pass is not yet initialized"));
    }

    // load the index if already exists
    let mut index_list: Vec<(Uuid, String)> = index::get_index().unwrap_or(Vec::new());

    // from the root folder, recursively walk all files and ask for the indices.
    let to_index = walk_recursively(root_folder.as_ref(), force)?;

    if to_index.len() == 0 {
        // no keys to index! check if the index file exists
        if index_list.len() == 0 {
            println!("Generating an empty index file!");
            index::write(&index_list)?;
        } else {
            println!("Nothing to do!");
        }
        return Ok(());
    }

    match utils::confirm(format!("\nGenerating index for {} keys! Do you wish to continue?", to_index.len()), false) {
        false => return Err(Error::new(ErrorKind::Interrupted, "Action interrupted!")),
        true => {}
    }

    for key_filename in to_index {

        let key_name = key_filename[root_folder_len..].to_string();
        println!("Indexing {}", key_name);

        // get the entry
        let mut e: Entry = Entry::from_path(key_name.clone())?;

        // check if the entry does already end with a newline
        if !e.raw.ends_with('\n') {
            e.raw.push('\n');
        }

        // check if the path is already set
        if e.path.is_some() {
            // check if the path is already set correctly
            
        } else {
            // add path key
            e.path = Some(key_name.clone());
            e.raw.push_str(def::PATH_KEY);
            e.raw.push_str(key_name.as_str());
            e.raw.push('\n');
        }

        // check if an uuid is already present
        if e.uuid == Uuid::nil() {
            // No uuid is present! generate one and write
            e.uuid = Uuid::new_v4();
            e.raw.push_str(def::UUID_KEY);
            e.raw.push_str(format!("{}", e.uuid).as_str());
            e.raw.push('\n');
        }

        // write the new entry
        e.write()?;

        // add the entry to the index file
        index_list.push((e.uuid, key_name));
    }

    // update the index list
    index::write(&index_list)

}

fn walk_recursively(dir: &Path, force: bool) -> Result<Vec<String>, Error> {
    let mut res: Vec<String> = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // handle directory

                // skip git folder
                if path.ends_with(def::GIT_FOLDER) { continue }
                if path.ends_with(def::UUID_FOLDER) { continue }

                // ask to change to add all, to ask again or to skip the directory
                let force_child = match force {
                    true => true,
                    false => match skip_ask_all(&path) {
                        Some(true) => true,
                        Some(false) => false,
                        None => continue
                    }
                };

                // call calk_recursive recursively
                res.append(&mut walk_recursively(&path, force_child)?);

            } else {
                // handle files
                match path.extension() {
                    Some(ext) => {
                        if ext == "gpg" {
                            if force || utils::confirm(format!("Index {}:", path.display()), false) {
                                res.push(path.to_str().unwrap().to_string());
                            }
                        }
                    }
                    None => {}
                }
            }
        }
    }

    Ok(res)
}


fn skip_ask_all(path: &Path) -> Option<bool> {
    print!("Index {}: [f]orce, [s]kip or [A]sk: ", path.display());
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    if answer == "f" || answer == "F" {
        Some(true)
    } else if answer == "s" || answer == "S" {
        None
    } else {
        Some(false)
    }
}
