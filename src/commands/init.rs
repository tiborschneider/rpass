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
use std::io;
use std::io::prelude::*;
use std::path::Path;

use dirs::home_dir;
use text_io::read;
use uuid::Uuid;

use crate::commands::utils;
use crate::config::CFG;
use crate::def;
use crate::errors::{Error, Result};
use crate::pass::entry::Entry;
use crate::pass::index;

pub fn init(force: bool) -> Result<()> {
    let mut root_folder = home_dir().unwrap();
    root_folder.push(def::ROOT_FOLDER);
    let root_folder_len: usize = root_folder.as_path().to_str().unwrap().len() + 1;

    // check if the password store folder exists
    if !root_folder.is_dir() {
        // pass is not yet initialized!
        println!("[Error] Pass is not yet initialized! Initialize it by running \"pass init\"!");
        return Err(Error::Other("pass is not yet initialized".to_string()));
    }

    // load the index if already exists
    let mut index_list: Vec<(Uuid, String)> = index::get_index().unwrap_or_else(|_| Vec::new());

    // from the root folder, recursively walk all files and ask for the indices.
    let to_index = walk_recursively(root_folder.as_ref(), force)?;

    if to_index.is_empty() {
        // no keys to index! check if the index file exists
        if index_list.is_empty() {
            println!("Generating an empty index file!");
            index::write(&index_list)?;
        } else {
            println!("Nothing to do!");
        }
        return Ok(());
    }

    if !utils::confirm(
        format!(
            "\nGenerating index for {} keys! Do you wish to continue?",
            to_index.len()
        ),
        false,
    ) { return Err(Error::Interrupted) }

    for key_filename in to_index {
        let key_name = key_filename[root_folder_len..]
            .trim_end_matches(".gpg")
            .to_string();
        println!("Indexing {}", key_name);

        // get the entry
        let mut e: Entry = Entry::from_path(&key_name)?;

        // check if the entry does already end with a newline
        if !e.raw.ends_with('\n') {
            e.raw.push('\n');
        }

        // check if the path is already set correctly
        if match &e.path {
            Some(p) => key_name == *p,
            None => false,
        } {
            e.path = Some(key_name.clone());
        }

        // check if an uuid is already present
        if e.uuid == Uuid::nil() {
            e.uuid = Uuid::new_v4();
        }

        // write the new entry
        e.write()?;

        // add the entry to the index file
        index_list.push((e.uuid, key_name));
    }

    // update the index list
    index::write(&index_list)
}

fn walk_recursively(dir: &Path, force: bool) -> Result<Vec<String>> {
    let mut res: Vec<String> = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // handle directory

                // skip git folder
                if path.ends_with(def::GIT_FOLDER) {
                    continue;
                }
                if path.ends_with(CFG.main.uuid_folder) {
                    continue;
                }
                if path.ends_with(CFG.main.sync_folder) {
                    continue;
                }

                // ask to change to add all, to ask again or to skip the directory
                let force_child = match force {
                    true => true,
                    false => match skip_ask_all(&path) {
                        Some(true) => true,
                        Some(false) => false,
                        None => continue,
                    },
                };

                // call calk_recursive recursively
                res.append(&mut walk_recursively(&path, force_child)?);
            } else {
                // handle files
                if let Some(ext) = path.extension() {
                    if ext == "gpg"
                        && (force || utils::confirm(format!("Index {}:", path.display()), false))
                    {
                        res.push(path.to_str().unwrap().to_string());
                    }
                }
            }
        }
    }

    Ok(res)
}

fn skip_ask_all(path: &Path) -> Option<bool> {
    print!("Index {}: [f]orce, [s]kip or [A]sk: ", path.display());
    io::stdout().flush().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    if answer == "f" || answer == "F" {
        Some(true)
    } else if answer == "s" || answer == "S" {
        None
    } else {
        Some(false)
    }
}
