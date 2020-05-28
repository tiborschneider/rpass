use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process::Command;
use std::{thread, time};

use dirs::home_dir;
use ctrlc;

use crate::def;

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
    sync_path.push(def::SYNC_FOLDER);

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
