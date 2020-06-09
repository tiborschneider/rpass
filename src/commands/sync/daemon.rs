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

use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process::Command;
use std::{thread, time};

use dirs::home_dir;
use ctrlc;

use crate::def;
use crate::config::CFG;

pub fn daemon() -> Result<(), Error> {

    // start a handler for ctrlc
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // start the ssdh daemon
    println!("Starting sshd...");
    Command::new("sudo")
        .arg("systemctl")
        .arg("start")
        .arg("sshd")
        .spawn()?
        .wait()?;

    // wait for one second
    thread::sleep(time::Duration::from_millis(1000));
    
    // get sync path
    let mut sync_path = home_dir().unwrap();
    sync_path.push(def::ROOT_FOLDER);
    sync_path.push(CFG.main.sync_folder);

    // pull and push changes to local repository
    println!("pushing changes from local repository");
    Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg("master")
        .current_dir(&sync_path)
        .spawn()?
        .wait()?;
    Command::new("git")
        .arg("push")
        .arg("origin")
        .arg("master")
        .current_dir(&sync_path)
        .spawn()?
        .wait()?;

    println!("\nDaemon is running. press Ctrl-C to stop daemon");
    while running.load(Ordering::SeqCst) {}

    println!("pulling new changes from local repository");
    Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg("master")
        .current_dir(&sync_path)
        .spawn()?
        .wait()?;

    println!("Stopping sshd...");
    Command::new("sudo")
        .arg("systemctl")
        .arg("stop")
        .arg("sshd")
        .spawn()?
        .wait()?;

    Ok(())
}
