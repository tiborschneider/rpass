use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::fs::{self, File};
use std::process::Command;
use std::str;

use dirs::home_dir;
use unidiff::{self, PatchSet};
use uuid::Uuid;

use crate::def;
use crate::pass::index;
use crate::pass::entry::Entry;
use crate::commands::sync::update_sync_commit_file;

// TODO also sync the other way!

pub fn sync(apply: bool) -> Result<(), Error> {

    let mut slave_changes = false;

    println!("Loading diffs...");
    let (master_patch, slave_patch) = parse_diffs()?;

    let index_list = index::get_index()?;
    let index_path_map = index::to_hashmap(&index_list);
    let index_uuid_map = index::to_hashmap_reverse(&index_list);

    //-----------------
    // MASTER ==> SLAVE
    //-----------------

    // Step 1, copy all new master passwords over to the slave
    for new_file in master_patch.added_files() {
        if !new_file.target_file.contains(def::UUID_FOLDER) { continue }
        if !new_file.target_file.starts_with("b/") { continue }
        if !new_file.target_file.ends_with(".gpg") { continue }
        let uuid = uuid_from_diff_filename(&new_file.target_file)?;
        let path = index_path_map[&uuid];

        println!("New entry    [M -> S]: {}", path);
        if apply {
            move_entry_to_slave(uuid, path, false)?;
            slave_changes = true;
        }
    }

    // step 2: apply deleted entries from the master to the slave
    for old_file in master_patch.removed_files() {
        if !old_file.source_file.contains(def::UUID_FOLDER) { continue }
        if !old_file.source_file.starts_with("a/") { continue }
        if !old_file.source_file.ends_with(".gpg") { continue }
        let uuid = uuid_from_diff_filename(&old_file.source_file)?;

        // extract the path from the diff
        let path = match old_file.into_iter().flatten().filter(|l| l.value.starts_with(def::PATH_KEY)).next() {
            Some(l) => String::from(&l.value[def::PATH_KEY.len()..]),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("Entry ({}) to delete did not contain a path key!", uuid)))
        };

        println!("Remove entry [M -> S]: {}", path);

        if apply {
            remove_slave_entry(&path)?;
            slave_changes = true;
        }
    }

    // step 3: apply all changes of the master on the slave
    for mod_file in master_patch.modified_files() {
        if !mod_file.target_file.contains(def::UUID_FOLDER) { continue }
        if !mod_file.target_file.starts_with("b/") { continue }
        if !mod_file.target_file.ends_with(".gpg") { continue }
        if mod_file.target_file.contains(def::INDEX_FILE) { continue }
        let uuid = uuid_from_diff_filename(&mod_file.target_file)?;
        let path = index_path_map[&uuid];

        // Check wether the path line was changed
        match mod_file.into_iter().flatten().filter(|l| l.line_type == "-" && l.value.starts_with(def::PATH_KEY)).next() {
            Some(l) => {
                let old_path = &l.value[def::PATH_KEY.len()..];
                println!("Rename entry [M -> S]: {} -> {}", old_path, path);
                if apply {
                    rename_slave_entry(old_path, path)?;
                    slave_changes = true;
                }
            },
            None => {} // no move necessary
        }

        // copy over the new file
        println!("Modify entry [M -> S]: {}", path);
        if apply {
            move_entry_to_slave(uuid, path, true)?;
            slave_changes = true;
        }
    }

    //-----------------
    // MASTER <== SLAVE
    //-----------------

    // step 1: Remove entries in the master
    for old_file in slave_patch.removed_files() {
        if !old_file.source_file.starts_with("a/") { continue }
        if !old_file.source_file.ends_with(".gpg") { continue }
        let path = path_from_slave_diff_filename(&old_file.source_file);

        println!("Remove entry [M <- S]: {}", path);

        // check if the uuid exists and is indexed
        if !index_uuid_map.contains_key(path.as_str()) {
            return Err(Error::new(ErrorKind::NotFound, "The slave entry does not exist in the index!"))
        }

        let uuid = index_uuid_map[path.as_str()];

        if apply {
            index::remove(uuid)?;
        }
    }

    // step 2: Add new entries to the master
    for new_file in slave_patch.added_files() {
        if !new_file.target_file.starts_with("b/") { continue }
        if !new_file.target_file.ends_with(".gpg") { continue }
        let path = path_from_slave_diff_filename(&new_file.target_file);
        let full_path = format!("{}/{}", def::SYNC_FOLDER, path);

        println!("Add entry    [M <- S]: {}", path);

        if apply {
            // get the entry
            let mut e: Entry = Entry::from_path(&full_path)?;

            // set path and uuid
            e.path = Some(path.clone());
            e.uuid = Uuid::new_v4();

            // write the new entry
            e.create()?;

            // now, we must change the entry of the slave, to reflect our changes (else, both branches would diverge)
            move_entry_to_slave(e.uuid, &path, true)?;
            slave_changes = true;
        }
    }

    // step 3: Entry was edited by the slave, apply changes to the master
    for mod_file in slave_patch.modified_files() {
        if !mod_file.target_file.starts_with("b/") { continue }
        if !mod_file.target_file.ends_with(".gpg") { continue }
        let path = path_from_slave_diff_filename(&mod_file.target_file);
        let full_path = format!("{}/{}", def::SYNC_FOLDER, path);

        println!("Modify entry [M <- S]: {}", path);

        // check if entry already exists in the index
        if !index_uuid_map.contains_key(path.as_str()) {
            return Err(Error::new(ErrorKind::NotFound, "The slave entry does not exist in the index!"))
        }

        let uuid = index_uuid_map[path.as_str()];

        if apply {
            // get the entry
            let e: Entry = Entry::from_path(&full_path)?;

            // check if everything is ok
            if e.uuid != uuid {
                return Err(Error::new(ErrorKind::InvalidData, "Slave has modified the uuid!"))
            }
            if e.path.as_ref() != Some(&path) {
                return Err(Error::new(ErrorKind::InvalidData, "Slave has an invalid path!"))
            }

            // write the changes
            e.write()?;
        }
    }

    //-----------------
    // SALVE GIT COMMIT
    //-----------------

    if slave_changes {
        // change working directory to the sync folder
        let mut working_path = home_dir().unwrap();
        working_path.push(def::ROOT_FOLDER);
        working_path.push(def::SYNC_FOLDER);

        // add changes and fcommit
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&working_path)
            .spawn()?
            .wait()?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("rpass sync")
            .current_dir(&working_path)
            .spawn()?
            .wait()?;
    }

    // update the commits file
    if apply {
        update_sync_commit_file()?;
    }

    Ok(())
}

fn uuid_from_diff_filename(diff_filename: &String) -> Result<Uuid, Error> {
    let uuid_start = "b//".len() + def::UUID_FOLDER.len();
    let uuid_end = diff_filename.len() - ".gpg".len();
    let uuid_slice = &diff_filename[uuid_start..uuid_end];
    match Uuid::parse_str(uuid_slice) {
        Ok(id) => Ok(id),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "Invalid UUID while parsing name of added file in master patch"))
    }
}

fn path_from_slave_diff_filename(diff_filename: &String) -> String {
    let path_start = "b/".len();
    let path_end = diff_filename.len() - ".gpg".len();
    return String::from(&diff_filename[path_start..path_end])
}

fn move_entry_to_slave(uuid: Uuid, path: &str, overwrite: bool) -> Result<(), Error> {
    let mut working_path = home_dir().unwrap();
    working_path.push(def::ROOT_FOLDER);
    let mut src_path = working_path.clone();
    src_path.push(def::UUID_FOLDER);
    src_path.push(format!("{}.gpg", uuid));
    let mut dst_path = working_path;
    dst_path.push(def::SYNC_FOLDER);
    dst_path.push(format!("{}.gpg", path));

    let parent = dst_path.parent().unwrap();
    if !parent.is_dir() {
        fs::create_dir_all(parent)?;
    }

    // if overwrite is not set, we must create a new file, and thus, the dst_path is not allowed to exist already.
    if dst_path.is_file() && !overwrite {
        return Err(Error::new(ErrorKind::AlreadyExists, "Slave already has an entry at the given location!"))
    }
    // if overwrite is set, we must edit the file, and thus, the dst_path must already exist
    if !dst_path.is_file() && overwrite {
        return Err(Error::new(ErrorKind::NotFound, "Cannot modify slave entry, entry does not exist!"))
    }

    // copy the file over
    fs::copy(src_path, dst_path)?;

    Ok(())
}

fn remove_slave_entry(path: &str) -> Result<(), Error> {
    let mut dst_path = home_dir().unwrap();
    dst_path.push(def::ROOT_FOLDER);
    dst_path.push(def::SYNC_FOLDER);
    dst_path.push(format!("{}.gpg", path));

    // remove the file
    match fs::remove_file(&dst_path) {
        Ok(()) => {},
        Err(e) => match e.kind() {
            ErrorKind::NotFound => println!("Warning: Entry {} does not exist for the slave!", path),
            _ => return Err(e)
        }
    }

    // recursively walk back directories if the current path is empty
    loop {
        dst_path.pop();
        if dst_path.file_name().unwrap() == def::SYNC_FOLDER { break }
        match fs::remove_dir(&dst_path) {
            Ok(()) => {},
            Err(e) => match e.kind() {
                ErrorKind::Other => break,
                _ => return Err(e)
            }
        }
    }

    Ok(())
}

fn rename_slave_entry(old_path: &str, new_path: &str) -> Result<(), Error> {
    let mut working_path = home_dir().unwrap();
    working_path.push(def::ROOT_FOLDER);
    working_path.push(def::SYNC_FOLDER);
    let mut src_path = working_path.clone();
    src_path.push(format!("{}.gpg", old_path));
    let mut dst_path = working_path;
    dst_path.push(format!("{}.gpg", new_path));

    // create target directory if it does not already exist
    let parent = dst_path.parent().unwrap();
    if !parent.is_dir() {
        fs::create_dir_all(parent)?;
    }

    // check if the new_file already exists
    if dst_path.is_file() {
        return Err(Error::new(ErrorKind::AlreadyExists, "Destination file already exists!"))
    }

    // move the entry
    fs::rename(src_path, dst_path)
}

fn parse_diffs() -> Result<(PatchSet, PatchSet), Error> {
     let (master_commit, slave_commit) = get_last_sync_commits()?;

    // delete old file if it exists
    let mut working_path = home_dir().unwrap();
    working_path.push(def::ROOT_FOLDER);

    // get master commit
    let master_patch = Command::new("git")
        .arg("diff")
        .arg(master_commit)
        .arg("--no-renames")
        .current_dir(&working_path)
        .output()?
        .stdout;

    working_path.push(def::SYNC_FOLDER);

    // get slave commit
    let slave_patch = Command::new("git")
        .arg("diff")
        .arg(slave_commit)
        .arg("--no-renames")
        .current_dir(&working_path)
        .output()?
        .stdout;

    // parse the commit string
    let mut master_patchset = PatchSet::new();
    match master_patchset.parse(str::from_utf8(&master_patch).unwrap()) {
        Ok(()) => {},
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, format!("{:?}", e)))
    };

    let mut slave_patchset = PatchSet::new();
    match slave_patchset.parse(str::from_utf8(&slave_patch).unwrap()) {
        Ok(()) => {},
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, format!("{:?}", e)))
    };

    Ok((master_patchset, slave_patchset))
}

fn get_last_sync_commits() -> Result<(String, String), Error> {

    let mut sync_commit_file = home_dir().unwrap();
    sync_commit_file.push(def::ROOT_FOLDER);
    sync_commit_file.push(def::SYNC_FOLDER);
    sync_commit_file.push(def::SYNC_COMMIT_FILE);

    // read the file
    let file = File::open(sync_commit_file)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    let master_commit = match lines.next() {
        Some(s) => s?,
        None => return Err(Error::new(ErrorKind::InvalidData, ".sync_commit file is invalid!"))
    };

    let slave_commit = match lines.next() {
        Some(s) => s?,
        None => return Err(Error::new(ErrorKind::InvalidData, ".sync_commit file is invalid!"))
    };

    if master_commit.len() != 40 || slave_commit.len() != 40 {
        Err(Error::new(ErrorKind::InvalidData, ".sync_commit file is invalid!"))
    } else {
        Ok((master_commit, slave_commit))
    }

}
