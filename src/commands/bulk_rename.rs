// rpass: a password manager based on pass, written in rust
// Copyright (C) 2021, Tibor Schneider
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

use crate::commands::utils;
use crate::config::CFG;
use crate::def;
use crate::errors::Result;
use crate::pass::entry::Entry;

use dirs::home_dir;
use uuid::Uuid;

use std::collections::HashMap;
use std::fs::copy;
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;

pub fn bulk_rename() -> Result<()> {
    // extract the correct file names
    let mut uuid_folder = home_dir().unwrap();
    uuid_folder.push(def::ROOT_FOLDER);
    uuid_folder.push(CFG.main.uuid_folder);

    let mut index_file = uuid_folder.clone();
    index_file.push(CFG.main.index_file);
    let index_entry = format!("{}/{}", CFG.main.uuid_folder, CFG.main.index_entry);

    let mut shadow_file = uuid_folder;
    shadow_file.push(".shadow.gpg");
    let shadow_entry = format!("{}/.shadow", CFG.main.uuid_folder);

    // first of all, copy the index file to index_clone
    copy(index_file, shadow_file)?;

    // then, start the editor to edit the shadow file
    Command::new("pass")
        .arg("edit")
        .arg(format!("{}/{}", CFG.main.uuid_folder, ".shadow"))
        .spawn()?
        .wait()?;

    // now, we can compare both files and get all keys that have changed
    let index = String::from_utf8(Command::new("pass").arg(index_entry).output()?.stdout)?;
    let shadow = String::from_utf8(Command::new("pass").arg(shadow_entry).output()?.stdout)?;
    let index: HashMap<Uuid, String> = parse_index_file(&index);
    let mut shadow: HashMap<Uuid, String> = parse_index_file(&shadow);

    let modified_uuids: Vec<Uuid> = index
        .iter()
        .filter(|(k, v)| shadow.contains_key(k) && shadow.get(k) != Some(v))
        .map(|(k, _)| *k)
        .collect();

    if modified_uuids.is_empty() {
        println!("\nNo keys are renamed!");
        return Ok(());
    }

    println!("\nThe following modifications will be performed:\n");
    for uuid in modified_uuids.iter() {
        println!(
            "    {} --> {}",
            index.get(uuid).unwrap(),
            shadow.get(uuid).unwrap()
        );
    }

    if !utils::confirm("\nDo you want to continue?", false) {
        println!("Operation cancelled!");
        return Ok(());
    }

    // perform the renaming
    print!("working");
    io::stdout().flush()?;
    for uuid in modified_uuids {
        Entry::get(uuid)?.change_path(shadow.remove(&uuid).unwrap())?;
        print!(".");
        io::stdout().flush()?;
    }
    println!(" done!");

    Ok(())
}

fn parse_index_file(input: &str) -> HashMap<Uuid, String> {
    input
        .lines()
        .filter_map(|l| -> Option<(&str, &str)> { l.split_once(' ') })
        .filter_map(|(uuid, path)| Uuid::from_str(uuid).ok().map(|u| (u, path.to_string())))
        .collect()
}
