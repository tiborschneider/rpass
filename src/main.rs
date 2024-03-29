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

use std::{
    process::exit,
    sync::atomic::{AtomicBool, Ordering},
};

use clap::{App, Arg, SubCommand};

mod commands;
mod config;
mod def;
mod errors;
mod pass;
mod rofi_app;

use config::CFG;
use errors::{Error, Result};
use rofi::{Rofi, RofiChild, Width};

const DEFAULT_PW_SIZE: usize = 20;
static ROFI_APP: AtomicBool = AtomicBool::new(false);

fn main() {
    let matches = App::new("rpass")
        .version("0.3.1")
        .author("Tibor Schneider <tiborschneider@bluewin.ch>")
        .about("Manage pass without leaking information")
        .subcommand(
            SubCommand::with_name("menu")
                .about("Interactive app with rofi interface")
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Initializes rpass and start the migration.")
                .arg(Arg::with_name("force")
                     .short("f")
                     .long("force")
                     .help("automatically adds all entries to the index, without asking")
                     .takes_value(false))
        )
        .subcommand(
            SubCommand::with_name("interactive")
                .about("Copy username or password to clipboard using interactive dmenu")
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Print all entry information")
                .arg(Arg::with_name("path")
                     .short("d")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key to show content")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key to show content")
                     .takes_value(true)
                     .conflicts_with("path"))
                .arg(Arg::with_name("password")
                     .short("p")
                     .long("password")
                     .help("only print out the password")
                     .takes_value(false))
                .arg(Arg::with_name("username")
                     .short("n")
                     .long("username")
                     .help("only print out the username")
                     .takes_value(false))
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Edit content of entry")
                .arg(Arg::with_name("path")
                     .short("p")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key to eidt")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key to edit")
                     .takes_value(true)
                     .conflicts_with("path"))
        )
        .subcommand(
            SubCommand::with_name("mv")
                .about("Rename a specific key")
                .arg(Arg::with_name("path")
                     .short("p")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key")
                     .takes_value(true)
                     .conflicts_with("path"))
                .arg(Arg::with_name("dst")
                     .short("d")
                     .long("dst")
                     .value_name("DESTINATION")
                     .help("Path to move the old key to")
                     .takes_value(true))
        )
        .subcommand(
            SubCommand::with_name("passwd")
                .about("Change password of a specific key")
                .arg(Arg::with_name("path")
                     .short("d")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key")
                     .takes_value(true)
                     .conflicts_with("path"))
                .arg(Arg::with_name("password")
                     .short("p")
                     .long("password")
                     .value_name("DESTINATION")
                     .help("new password to set")
                     .takes_value(true))
                .arg(Arg::with_name("generate")
                     .short("g")
                     .long("generate")
                     .help("automatically generate a password with 20 characters")
                     .takes_value(false)
                     .conflicts_with("password"))
        )
        .subcommand(
            SubCommand::with_name("insert")
                .about("Insert a new key")
                .arg(Arg::with_name("path")
                     .short("d")
                     .long("path")
                     .value_name("PATH")
                     .help("path under which to store the new key")
                     .takes_value(true))
                .arg(Arg::with_name("username")
                     .short("u")
                     .long("user")
                     .value_name("USERNAME")
                     .help("username for the key")
                     .takes_value(true))
                .arg(Arg::with_name("password")
                     .short("p")
                     .long("password")
                     .value_name("PASSWORD")
                     .help("password for the key")
                     .takes_value(true))
                .arg(Arg::with_name("generate")
                     .short("g")
                     .long("generate")
                     .help("automatically generate a password with 20 characters")
                     .takes_value(false)
                     .conflicts_with("password"))
                .arg(Arg::with_name("url")
                     .long("url")
                     .value_name("URL")
                     .help("url for the key")
                     .takes_value(true))
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Delete an existing key")
                .arg(Arg::with_name("path")
                     .short("d")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key to delete")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key to delete")
                     .takes_value(true)
                     .conflicts_with("path"))
                .arg(Arg::with_name("force")
                     .short("f")
                     .long("force")
                     .help("skip confirmation")
                     .takes_value(false))
        )
        .subcommand(
            SubCommand::with_name("ls")
                .about("Lists all keys in a tree-like structure")
        )
        .subcommand(
            SubCommand::with_name("fix-index")
                .about("Checks all indices and fixes them")
        )
        .subcommand(
            SubCommand::with_name("sync")
                .about("Synchronize repository in non-uuid format. Without subcommand, sync local repos and start daemon.")
                // .subcommand(
                //     SubCommand::with_name("repo")
                //         .about("Synchronizes the master and slave repository.")
                //         .arg(Arg::with_name("debug")
                //              .short("d")
                //              .long("debug")
                //              .help("Only show changes, without applying them")
                //              .takes_value(false))
                // )
                .subcommand(
                    SubCommand::with_name("init")
                        .about("Initializes sync framework")
                )
                // .subcommand(
                //     SubCommand::with_name("daemon")
                //         .about("Starts the daemon for synchronization in the local network")
                // )
        )
        .subcommand(
            SubCommand::with_name("default-config")
                .about("Write the default config to disk")
        )
        .subcommand(
            SubCommand::with_name("bulk-rename")
                .about("Rename multiple keys at the same time, using $EDITOR.

The bulk renaming is performed by opening a copy of the index key in $EDITOR.
After performing all modifications, the script will search for all uuids for
which the path was changed. If one uuid is changed, or a line is deleted, then
the respective entry will be ignored. Only entries for which the modified and
original uuid are identical will be compared.

Before performing all modifications, the script will ask the user to confirm the
actions.")
        )
        .get_matches();

    let result = match matches.subcommand() {
        ("menu", _) => {
            ROFI_APP.store(true, Ordering::Relaxed);
            rofi_app::rofi_app()
        },
        ("init", Some(args)) => commands::init(args.is_present("force")),
        ("interactive", _) => {
            ROFI_APP.store(true, Ordering::Relaxed);
            commands::interactive()
        },
        ("get", Some(args)) => commands::get(
            args.value_of("path"),
            args.value_of("uuid"),
            false,
            args.is_present("password"),
            args.is_present("username"),
        ),
        ("edit", Some(args)) => commands::edit(args.value_of("path"), args.value_of("uuid"), false),
        ("mv", Some(args)) => commands::mv(
            args.value_of("path"),
            args.value_of("uuid"),
            args.value_of("dst"),
            false,
        ),
        ("insert", Some(args)) => commands::insert(
            args.value_of("path"),
            args.value_of("username"),
            args.value_of("password"),
            args.value_of("url"),
            match args.is_present("generate") {
                true => Some(DEFAULT_PW_SIZE),
                false => None,
            },
            false,
        ),
        ("passwd", Some(args)) => commands::passwd(
            args.value_of("path"),
            args.value_of("uuid"),
            args.value_of("password"),
            match args.is_present("generate") {
                true => Some(DEFAULT_PW_SIZE),
                false => None,
            },
            false,
        ),
        ("rm", Some(args)) => commands::delete(
            args.value_of("path"),
            args.value_of("uuid"),
            args.is_present("force"),
            false,
        ),
        ("ls", _) => commands::list(),
        ("fix-index", _) => commands::fix_index(),
        ("sync", Some(args)) => match args.subcommand() {
            // ("repo", Some(a)) => commands::sync::sync(!a.is_present("debug")),
            ("init", _) => commands::sync::init(),
            // ("daemon", _) => commands::sync::daemon(),
            _ => commands::sync::full(),
        },
        ("default-config", _) => config::store_config(),
        ("bulk-rename", _) => commands::bulk_rename(),
        _ => {
            ROFI_APP.store(true, Ordering::Relaxed);
            rofi_app::rofi_app()
        },
    };

    match result {
        Ok(()) => {}
        Err(e) => match e {
            Error::Interrupted => {}
            _ => {
                eprintln!("Error: {:#?}", e);
                exit(1);
            }
        },
    }
}

pub struct Loading(Option<RofiChild<usize>>);

#[allow(dead_code)]
impl Loading {
    pub fn new(msg: impl Into<String>) -> Result<Self> {
        Ok(Self(if ROFI_APP.load(Ordering::Relaxed) {
            let options = vec![
                def::format_small("please_wait"),
            ];

            Some(
                Rofi::new(&options)
                    .prompt(msg)
                    .pango()
                    .theme(CFG.theme.theme_name)
                    .width(Width::Pixels(500))?
                    .spawn_index()?,
            )
        } else {
            eprintln!("{}", msg.into());
            None
        }))
    }

    pub fn done(mut self) -> Result<()> {
        if let Some(child) = self.0.as_mut() {
            child.kill()?
        }
        Ok(())
    }
}

impl Drop for Loading {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
        }
    }
}
