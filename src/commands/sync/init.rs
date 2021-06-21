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

use std::fs::{self, File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::process::Command;

use dirs::home_dir;

use crate::commands::sync::update_sync_commit_file;
use crate::config::CFG;
use crate::def;
use crate::errors::{Error, Result};
use crate::pass::index;

pub fn init() -> Result<()> {
    // setup gitignore in main pass repo
    init_gitignore()?;
    // setup .sync repo
    init_snyc_folder()?;
    // copy all keys over
    do_initial_sync()?;
    // update the sync_commit file
    update_sync_commit_file()?;

    println!(
        "
Now, you need to setup a git server locally on this machine using ssh. To do so,
you need to perform the following steps:

1. Install sshd and git
2. Setup a git user, following the instructions here:
   https://git-scm.com/book/en/v2/Git-on-the-Server-Setting-Up-the-Server
   Make sure the user has a home directory, and setup the authorized keys
   properly. At least, add the key of your user and the one of the target
   device (android phone).
3. Login as git user:
       sh -s /bin/bash git
4. Generate an empty and raw repository
       mkdir rpass.git
       cd rpass.git
       git init --bare
5. logout of git user (exit)
6. make sure to add the git user to ssh AllowedUsers
       sudo vim /etc/ssh/sshd_conf
   Add git user to the AllowedUsers.
7. add the origin in the rpass folder:
       cd ~/.password-store/.sync
       git remote add origin ssh://git@localhost/~git/rpass.git
8. Somehow get the gpg key to the mobile device

After those steps, you are done and you can run rpass sync daemon
"
    );

    Ok(())
}

fn init_gitignore() -> Result<()> {
    // check if .gitignore is created and contains the line .sync
    let mut git_changes = false;

    let mut gitignore_path = home_dir().unwrap();
    gitignore_path.push(def::ROOT_FOLDER);
    gitignore_path.push(".gitignore");
    if gitignore_path.is_file() {
        // .gitignore file already exists, check if the line .sync exists
        let mut line_found = false;
        {
            let gitignore_file = File::open(&gitignore_path)?;
            let reader = io::BufReader::new(gitignore_file);
            if reader
                .lines()
                .any(|l| l.as_ref().unwrap().starts_with(CFG.main.sync_folder))
            {
                line_found = true
            }
        }
        if !line_found {
            // append the line
            println!("Adding .sync to .gitignore!");
            let mut gitignore_file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(false)
                .open(&gitignore_path)?;
            gitignore_file.write_all(CFG.main.sync_folder.as_bytes())?;
            gitignore_file.write_all(b"\n")?;
            git_changes = true;
        }
    } else {
        // .gitignore file does not exist! create it
        println!("Generating .gitignore!");
        let mut gitignore_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&gitignore_path)?;
        gitignore_file.write_all(CFG.main.sync_folder.as_bytes())?;
        gitignore_file.write_all(b"\n")?;
        git_changes = true;
    }

    if git_changes {
        // commit the change of the .gitignore
        println!("committing changes!");
        Command::new("pass")
            .arg("git")
            .arg("add")
            .arg(".gitignore")
            .spawn()?
            .wait()?;
        Command::new("pass")
            .arg("git")
            .arg("commit")
            .arg("-m")
            .arg("added gitignore for sync")
            .spawn()?
            .wait()?;
    }

    Ok(())
}

fn init_snyc_folder() -> Result<()> {
    // generate .sync folder
    println!("Generating .sync folder!");
    let mut working_path = home_dir().unwrap();
    working_path.push(def::ROOT_FOLDER);
    working_path.push(CFG.main.sync_folder);
    if !working_path.is_dir() {
        fs::create_dir(&working_path)?;
    } else {
        println!("Error: the sync folder already exists! To re-initialize it, delete the sync folder entirely and try again!");
        return Err(Error::SyncError(".sync folder already exists"));
    }

    // initialize git repo
    println!("initializing git repo!");
    Command::new("git")
        .arg("init")
        .current_dir(&working_path)
        .output()?;

    // edit .git/config file
    println!("Edit gitconfig file");
    working_path.push(".git");
    working_path.push("config");
    {
        let mut config_file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(false)
            .open(&working_path)?;
        config_file.write_all(b"[diff \"gpg\"]\n")?;
        config_file.write_all(b"\tbinary = true\n")?;
        config_file.write_all(b"\ttextconv = gpg2 -d --quiet --yes --compress-algo=none --no-encrypt-to --batch --use-agent\n")?;
    }
    working_path.pop();
    working_path.pop();

    // generate the gitignore file
    println!("generating .gitignore!");
    working_path.push(".gitignore");
    {
        let mut gitignore_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&working_path)?;
        gitignore_file.write_all(CFG.main.sync_commit_file.as_bytes())?;
        gitignore_file.write_all(b"\n")?;
    }
    working_path.pop();

    // generate the gitattributes file
    println!("generating .gitattributes!");
    working_path.push(".gitattributes");
    {
        let mut attributes_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&working_path)?;
        attributes_file.write_all(b"*.gpg diff=gpg\n")?;
    }
    working_path.pop();

    // copy the gpg-id file
    println!("copying .gpg-id!");
    let mut main_gpg_id_path = working_path.clone();
    main_gpg_id_path.pop();
    main_gpg_id_path.push(".gpg-id");
    working_path.push(".gpg-id");
    fs::copy(main_gpg_id_path, &working_path)?;
    working_path.pop();

    // commit the change of the .gitignore
    println!("committing changes!");
    Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(&working_path)
        .spawn()?
        .wait()?;
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("initial commit")
        .current_dir(&working_path)
        .spawn()?
        .wait()?;

    Ok(())
}

fn do_initial_sync() -> Result<()> {
    let index_list = index::get_index()?;

    let mut sync_path = home_dir().unwrap();
    sync_path.push(def::ROOT_FOLDER);
    sync_path.push(CFG.main.sync_folder);

    let mut uuid_path = home_dir().unwrap();
    uuid_path.push(def::ROOT_FOLDER);
    uuid_path.push(CFG.main.uuid_folder);

    let mut git_changes = false;

    for (id, path) in index_list {
        // prepare destination folder
        let mut dst_path = sync_path.clone();
        dst_path.push(format!("{}.gpg", path));
        let parent = dst_path.parent().unwrap();
        if !parent.is_dir() {
            fs::create_dir_all(parent)?;
        }

        // copy the file over
        uuid_path.push(format!("{:?}.gpg", id));
        fs::copy(&uuid_path, dst_path)?;
        // reset uuid_path
        uuid_path.pop();

        git_changes = true;
    }

    if git_changes {
        // commit the change of the .gitignore
        println!("committing changes!");
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&sync_path)
            .spawn()?
            .wait()?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial sync")
            .current_dir(&sync_path)
            .spawn()?
            .wait()?;
    }

    Ok(())
}
