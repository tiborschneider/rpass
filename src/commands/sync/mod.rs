use std::io::Error;
use std::io::prelude::*;
use std::fs::{self, OpenOptions};
use std::process::Command;
use std::env;

use dirs::home_dir;

use crate::def;

mod sync;
mod daemon;
mod init;

pub use sync::sync;
pub use daemon::daemon;
pub use init::init;

pub fn full() -> Result<(), Error> {
    sync(true)?;
    daemon()?;
    sync(true)?;
    Ok(())
}

fn update_sync_commit_file() -> Result<(), Error> {

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

    working_path.push(def::SYNC_FOLDER);

    // get slave commit
    env::set_current_dir(&working_path)?;
    let slave_commit = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;

    working_path.push(def::SYNC_COMMIT_FILE);

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
