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

use std::io;
use std::io::prelude::*;
use std::clone::Clone;
use std::process::Command;
use std::{thread, time};

use rofi::{Rofi, Format, Width};
use uuid::Uuid;
use text_io::read;
use clipboard::{ClipboardProvider, ClipboardContext};
use notify_rust::{Notification, NotificationUrgency, Timeout};
use interactor;

use crate::errors::{Error, Result};
use crate::pass::index::{get_index, to_graph, to_hashmap_reverse};
use crate::pass::entry::Entry;
use crate::def;

pub fn choose_entry(path: Option<&str>, id: Option<&str>, use_rofi: bool) -> Result<Entry> {
    match (path, id) {
        (Some(path), None) => {
            let index_list = get_index()?;
            let uuid_lookup = to_hashmap_reverse(&index_list);
            let entry_id = match uuid_lookup.get(path) {
                Some(id) => id,
                None => return Err(Error::UnknownPath(path.to_string()))
            };
            Entry::get(entry_id.clone())
        },

        (None, Some(id)) => {
            let id = Uuid::parse_str(id)?;
            Entry::get(id)
        },
        
        (None, None) => {

            if use_rofi {
                choose_entry_rofi()
            } else {
                choose_entry_fzf()
            }

        },

        _ => panic!("This should not happen")
    }

}

fn choose_entry_fzf() -> Result<Entry> {
    let index_list = get_index()?;
    let index_list_clone = index_list.clone();
    let uuid_lookup = to_hashmap_reverse(&index_list_clone);
    let mut path_list: Vec<String> = index_list.into_iter().map(|x| x.1).collect();
    path_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    let choice = interactor::pick_from_list(Some(&mut Command::new("fzf")), &path_list, "")?;
    match uuid_lookup.get(choice.as_str()) {
        Some(id) => Entry::get(*id),
        None => Err(Error::UnknownPath(choice))
    }
}

fn choose_entry_rofi() -> Result<Entry> {
    // prepare the index list
    let index_list = get_index()?;
    let index_list_clone = index_list.clone();
    let uuid_lookup = to_hashmap_reverse(&index_list_clone);
    let mut path_list: Vec<String> = index_list.into_iter().map(|x| x.1).collect();
    path_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    let max_len = path_list.iter().map(|s| s.len()).fold(30, |cur, x| if x > cur {x} else {cur});

    // show with rofi
    let selection = Rofi::new(&path_list)
        .prompt("Select an entry")
        .pango()
        .lines(15)
        .width(Width::Characters(max_len))?
        .return_format(Format::StrippedText)
        .run()?;

    let entry_id = match uuid_lookup.get(selection.as_str()) {
        Some(id) => id,
        None => return Err(Error::UnknownPath(selection))
    };
    Entry::get(entry_id.clone())
}

pub fn gen_path_interactive() -> Result<String> {
    gen_path_recursive("".to_string())
}

pub fn gen_path_recursive(cur_path: String) -> Result<String> {

    let mut index_list = get_index().expect("Cannot get index file");
    index_list.sort_by(|a, b| b.1.to_lowercase().cmp(&a.1.to_lowercase()));
    let (g, root) = to_graph(&index_list);

    let mut last_node = root;

    for node in cur_path.split("/") {
        let mut walker = g.neighbors(last_node).detach();
        while let Some(child) = walker.next_node(&g) {
            if g.node_weight(child).unwrap() == &node {
                last_node = child;
                break;
            }
        }
    }

    let mut next_nodes: Vec<String> = Vec::new();
    next_nodes.push(def::format_button(def::DISPLAY_BTN_NEW_PATH));
    let mut walker = g.neighbors(last_node).detach();
    while let Some(child) = walker.next_node(&g) {
        if g.neighbors(child).count() >= 1 {
            next_nodes.push(g.node_weight(child).unwrap().to_string());
        }
    }

    if next_nodes.len() > 1 {
        let mut idx = Rofi::new(&next_nodes)
            .pango()
            .lines(15)
            .prompt(format!("Choose an entry: {}/", cur_path))
            .run_index()?;
        if idx == 0 {
            // Create new path
            ask_for_path(&cur_path)
        } else {
            idx = idx - 1;
            gen_path_recursive(format!("{}{}", cur_path, next_nodes[idx]))
        }
    } else {
        ask_for_path(&cur_path)
    }
}

fn ask_for_path(path: &String) -> Result<String> {
    let mut cur_path = path.clone();
    if cur_path.len() > 0 && !cur_path.ends_with("/") {
        cur_path.push_str("/");
    }
    let empty_options: Vec<String> = Vec::new();
    Ok(Rofi::new(&empty_options)
        .prompt(format!("Enter path: {}", cur_path))
        .return_format(Format::UserInput)
        .run()?)
    
}

pub fn confirm<S: AsRef<str>>(q: S, use_rofi: bool) -> bool {
    match use_rofi {
        true => confirm_rofi(q),
        false => confirm_stdio(q)
    }
}

fn confirm_stdio<S: AsRef<str>>(q: S) -> bool {
    print!("{} [y/N]: ", q.as_ref());
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    answer == "y" || answer == "Y"
}

fn confirm_rofi<S: AsRef<str>>(q: S) -> bool {
    let options = vec!["No".to_string(), "Yes".to_string()];
    match Rofi::new(&options).prompt(q.as_ref()).run() {
        Ok(s) => s == "Yes",
        Err(_) => false
    }
}

pub fn question<S: AsRef<str>>(q: S, use_rofi: bool) -> Result<Option<String>> {
    match use_rofi {
        true => question_rofi(q),
        false => question_stdio(q)
    }
}

fn question_stdio<S: AsRef<str>>(q: S) -> Result<Option<String>> {
    print!("{}: ", q.as_ref());
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    if answer.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(answer))
    }
}

pub fn question_rofi<S: AsRef<str>>(q: S) -> Result<Option<String>> {
    let options = vec![def::format_small(def::DISPLAY_EMPTY), def::format_small(def::DISPLAY_BTN_CANCEL)];
    let input = Rofi::new(&options)
        .prompt(format!("{}", q.as_ref()))
        .pango()
        .run()?;

    if input == "" || input == options[1] {
        Err(Error::Interrupted)
    } else if input == options[0] {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

pub fn two_options<S: AsRef<str>>(primary: S, secondary: S) -> bool {
    print!("1: {}, 2: {} [1|2]: ", primary.as_ref(), secondary.as_ref());
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    answer != "2"
}

pub fn copy_to_clipboard<S: AsRef<str>>(s: String, action: S, wait_for: Option<u64>) -> Result<()> {

    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(ctx) => ctx,
        Err(_) => return Err(Error::ClipboardError)
    };

    match ctx.set_contents(s) {
        Ok(_) => {},
        Err(_) => return Err(Error::ClipboardError)
    };

    let action_string = format!("Copied {}", action.as_ref());

    Notification::new()
        .summary(action_string.as_str())
        .urgency(NotificationUrgency::Normal)
        .timeout(Timeout::Milliseconds(5000))
        .show()?;

    match wait_for {
        Some(duration) => delayed_clipboard_clear(duration),
        None => Ok(())
    }
}

pub fn delayed_clipboard_clear(duration: u64) -> Result<()> {

    // wait for 5 seconds
    let ten_millis = time::Duration::from_millis(duration);
    thread::sleep(ten_millis);

    Ok(())
}

pub fn notify_error(e: Error) {
    match e {
        Error::Interrupted => {
            Notification::new()
                .summary("User interrupted action")
                .urgency(NotificationUrgency::Low)
                .timeout(Timeout::Milliseconds(4000))
                .show().unwrap();
        },
        Error::InvalidInput(s) => {
            Notification::new()
                .summary("Action failed!")
                .body(format!("User input is invalid:\n{}", s).as_str())
                .urgency(NotificationUrgency::Normal)
                .timeout(Timeout::Milliseconds(10000))
                .show().unwrap();
        },
        _ => {
            Notification::new()
                .summary("Action failed!")
                .body(format!("Error: {:?}", e).as_ref())
                .urgency(NotificationUrgency::Normal)
                .timeout(Timeout::Milliseconds(30000))
                .show().unwrap();
        }
    }
}

pub fn notify_action<S: AsRef<str>>(action: S) {
    Notification::new()
        .summary("Success!")
        .body(action.as_ref())
        .urgency(NotificationUrgency::Low)
        .timeout(Timeout::Milliseconds(4000))
        .show().unwrap();
}
