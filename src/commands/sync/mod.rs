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

use std::env;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::process::Command;

use dirs::home_dir;

use crate::config::CFG;
use crate::def;
use crate::errors::Result;

mod daemon;
mod init;
mod syncer;

pub use daemon::daemon;
pub use init::init;
pub use syncer::sync;

pub fn full() -> Result<()> {
    sync(true)?;
    daemon()?;
    sync(true)?;
    Ok(())
}

fn update_sync_commit_file() -> Result<()> {
    // current directory must be changed!
    let old_dir = env::current_dir()?;

    // delete old file if it exists
    let mut working_path = home_dir().unwrap();
    working_path.push(def::ROOT_FOLDER);

    // get master commit
    env::set_current_dir(&working_path)?;
    let master_commit = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;

    working_path.push(CFG.main.sync_folder);

    // get slave commit
    env::set_current_dir(&working_path)?;
    let slave_commit = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;

    working_path.push(CFG.main.sync_commit_file);

    if working_path.is_file() {
        fs::remove_file(&working_path)?;
    }

    // write the commit ids
    {
        let mut sync_commit_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&working_path)?;
        sync_commit_file.write_all(&master_commit)?;
        sync_commit_file.write_all(&slave_commit)?;
    }

    // set current directory back to the one before
    env::set_current_dir(old_dir)?;

    Ok(())
}
