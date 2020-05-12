use std::io;
use std::io::prelude::*;
use std::clone::Clone;
use std::fmt::Display;
use std::io::{Error, ErrorKind};
use std::{thread, time};

use rustofi::components::{ActionList, ItemList, EntryBox};
use rustofi::window::{Dimensions, Window};
use rustofi::RustofiResult;
use uuid::Uuid;
use text_io::read;
use clipboard::{ClipboardProvider, ClipboardContext};
use notify_rust::{Notification, NotificationUrgency, Timeout};

use crate::pass::index::{get_index, to_graph, to_hashmap_reverse};
use crate::pass::entry::Entry;
use crate::def;

pub fn rofi_display_item<'a, T: Display + Clone>(rofi: &mut ItemList<'a, T>, prompt: String, lines: usize) -> RustofiResult {
    let extra = vec!["".to_string(), def::PANGO_CANCEL_NAME.to_string()];
    let mut display_options: Vec<String> = rofi.items.iter().map(|s| s.clone().to_string()).collect();
    let num_lines: i32 = if lines > display_options.len() {display_options.len() as i32} else {lines as i32};
    display_options = display_options.into_iter().chain(extra.clone()).collect();
    let response = rofi
        .window
        .clone()
        .lines(num_lines)
        .prompt(prompt)
        .add_args(vec!("-i".to_string(), "-markup-rows".to_string()))
        .show(display_options.clone());
    match response {
        Ok(input) => {
            if input == def::PANGO_CANCEL_NAME || input == "" {
                RustofiResult::Cancel
            } else if input == " " {
                RustofiResult::Blank
            } else {
                for item in rofi.items.clone() {
                    if input == item.to_string() {
                        return (rofi.item_callback)(&item);
                    }
                }
                RustofiResult::Selection(input)
            }
        }
        Err(_) => RustofiResult::Error
    }
}

pub fn rofi_display_action<'a, T: Display + Clone>(rofi: &mut ActionList<'a, T>, prompt: String, lines: usize) -> RustofiResult {
    let extra = vec!["".to_string(), def::PANGO_CANCEL_NAME.to_string()];
    let mut display_options: Vec<String> = rofi.actions.iter().map(|s| s.to_string()).collect();
    let num_lines: i32 = if lines > display_options.len() {display_options.len() as i32} else {lines as i32};
    display_options = display_options.into_iter().chain(extra.clone()).collect();
    let response = rofi
        .window
        .clone()
        .lines(num_lines)
        .prompt(prompt)
        .add_args(vec!("-i".to_string(), "-markup-rows".to_string()))
        .show(display_options.clone());
    match response {
        Ok(input) => {
            if input == def::PANGO_CANCEL_NAME || input == "" {
                RustofiResult::Cancel
            } else if input == " " {
                RustofiResult::Blank
            } else {
                for action in rofi.actions.clone() {
                    if input == action.to_string() {
                        return (rofi.action_callback)(&rofi.item, &action.to_string());
                    }
                }
                RustofiResult::Action(input)
            }
        }
        Err(_) => RustofiResult::Error
    }
}

pub fn choose_entry(path: Option<&str>, id: Option<&str>) -> Result<Entry, Error> {
    match (path, id) {
        (Some(path), None) => {
            let index_list = get_index()?;
            let uuid_lookup = to_hashmap_reverse(&index_list);
            let entry_id = match uuid_lookup.get(path) {
                Some(id) => id,
                None => return Err(Error::new(ErrorKind::NotFound, "Path is not present in the index file!"))
            };
            Entry::get(entry_id.clone())
        },

        (None, Some(id)) => {
            let id = match Uuid::parse_str(id) {
                Ok(id) => id,
                Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot parse UUID"))
            };
            Entry::get(id)
        },
        
        (None, None) => {
            let index_list = get_index()?;
            let index_list_clone = index_list.clone();
            let uuid_lookup = to_hashmap_reverse(&index_list_clone);
            let mut path_list: Vec<String> = index_list.into_iter().map(|x| x.1).collect();
            path_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            let mut rofi = ItemList::new(path_list, Box::new(identity_callback));
            rofi.window = rofi.window.dimensions(Dimensions {width: 1000, height: 800, lines: 10, columns: 1});
            match rofi_display_item(&mut rofi, "Select an entry".to_string(), 10) {
                RustofiResult::Selection(s) => {
                    let entry_id = match uuid_lookup.get(s.as_str()) {
                        Some(id) => id,
                        None => return Err(Error::new(ErrorKind::NotFound, "Path is not present in the index file!"))
                    };
                    Entry::get(entry_id.clone())
                },
                RustofiResult::Blank        => Err(Error::new(ErrorKind::Interrupted, "User chose blank option")),
                RustofiResult::Cancel       => Err(Error::new(ErrorKind::Interrupted, "User chose cancel option")),
                RustofiResult::Exit         => Err(Error::new(ErrorKind::Interrupted, "User exited rofi")),
                _                           => Err(Error::new(ErrorKind::Other, "Rofi failed"))
            }
        },

        _ => panic!("This should not happen")
    }
}

pub fn identity_callback(name: &String) -> RustofiResult {
    RustofiResult::Selection(name.clone())
}

pub fn gen_path_interactive() -> Result<Option<String>, Error> {
    match gen_path_recursive("".to_string()) {
        RustofiResult::Selection(s) => Ok(Some(s)),
        RustofiResult::Action(_)    => Err(Error::new(ErrorKind::Other, "Rofi returned an action instead of a selection!")),
        RustofiResult::Success      => Err(Error::new(ErrorKind::UnexpectedEof, "Success returned without a string!")),
        RustofiResult::Blank        => Ok(None),
        RustofiResult::Error        => Err(Error::new(ErrorKind::Other, "Rofi returned unexpected error")),
        RustofiResult::Cancel       => Ok(None),
        RustofiResult::Exit         => Err(Error::new(ErrorKind::Interrupted, "Exit option chosen!")),
    }
}

pub fn gen_path_recursive(cur_path: String) -> RustofiResult {

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
    next_nodes.push(def::PANGO_NEW_PATH_NAME.to_string());
    let mut walker = g.neighbors(last_node).detach();
    while let Some(child) = walker.next_node(&g) {
        if g.neighbors(child).count() >= 1 {
            next_nodes.push(g.node_weight(child).unwrap().to_string());
        }
    }

    if next_nodes.len() > 1 {
        let mut rofi = ActionList::new(cur_path.clone(), next_nodes, Box::new(gen_path_callback));
        rofi_display_action(&mut rofi, format!("Choose an entry: {}/", cur_path), 10)
    } else {
        ask_for_path(&cur_path)
    }
}

fn gen_path_callback(path: &String, option: &String) -> RustofiResult {
    let mut cur_path = path.clone();
    if cur_path.len() > 0 {
        cur_path.push_str("/");
    }
    match option.as_str() {
        def::PANGO_NEW_PATH_NAME => ask_for_path(&cur_path),
        new_path => gen_path_recursive(format!("{}{}", cur_path, new_path))
    }
}

fn ask_for_path(path: &String) -> RustofiResult {
    let mut cur_path = path.clone();
    if cur_path.len() > 0 && !cur_path.ends_with("/") {
        cur_path.push_str("/");
    }
    let result = EntryBox::create_window()
        .prompt(format!("Enter path: {}", cur_path))
        .dimensions(Dimensions{width:1100, height:100, lines:1, columns:1})
        .show(vec!["".to_string()]);
    match result {
        Ok(input) => {
            if input == "" {
                RustofiResult::Cancel
            } else {
                RustofiResult::Selection(format!("{}{}", cur_path, input))
            }
        }
        Err(_) => RustofiResult::Error
    }
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
    match Window::new(q.as_ref()).format('s').add_args(vec!("-i".to_string())).show(options) {
        Ok(s) => s == "Yes",
        Err(_) => false
    }
}

pub fn question<S: AsRef<str>>(q: S, use_rofi: bool) -> Result<Option<String>, Error> {
    match use_rofi {
        true => question_rofi(q),
        false => question_stdio(q)
    }
}

fn question_stdio<S: AsRef<str>>(q: S) -> Result<Option<String>, Error> {
    print!("{}: ", q.as_ref());
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    if answer.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(answer))
    }
}

pub fn question_rofi<S: AsRef<str>>(q: S) -> Result<Option<String>, Error> {
    let result = EntryBox::create_window()
                .prompt(format!("{}", q.as_ref()))
                .lines(2)
                .dimensions(Dimensions{width:1100, height:100, lines:2, columns:1})
                .add_args(vec!("-i".to_string(), "-markup-rows".to_string()))
                .show(vec![def::PANGO_EMPTY_NAME.to_string(), def::PANGO_CANCEL_NAME.to_string()]);

    match result {
        Ok(input) => {
            if input == "" || input == def::PANGO_CANCEL_NAME {
                Err(Error::new(ErrorKind::Interrupted, "User interrupted question"))
            } else if input == def::PANGO_EMPTY_NAME {
                Ok(None)
            } else {
                Ok(Some(input))
            }
        }
        Err(_) => Err(Error::new(ErrorKind::Interrupted, "User interrupted question"))
    }
}

pub fn two_options<S: AsRef<str>>(primary: S, secondary: S) -> bool {
    print!("1: {}, 2: {} [1|2]: ", primary.as_ref(), secondary.as_ref());
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    answer != "2"
}

pub fn copy_to_clipboard<S: AsRef<str>>(s: String, action: S, wait_for: Option<u64>) -> Result<(), Error> {

    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(ctx) => ctx,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot generate clipboard context"))
    };

    match ctx.set_contents(s) {
        Ok(_) => {},
        Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot set clipboard content!"))
    };

    let action_string = format!("Copied {}", action.as_ref());

    Notification::new()
        .summary(action_string.as_str())
        .urgency(NotificationUrgency::Normal)
        .timeout(Timeout::Milliseconds(5000))
        .show().unwrap();

    match wait_for {
        Some(duration) => delayed_clipboard_clear(duration),
        None => Ok(())
    }
}

pub fn delayed_clipboard_clear(duration: u64) -> Result<(), Error> {

    // wait for 5 seconds
    let ten_millis = time::Duration::from_millis(duration);
    thread::sleep(ten_millis);

    Ok(())
}

pub fn notify_error(e: Error) {
    match e.kind() {
        ErrorKind::Interrupted => {
            Notification::new()
                .summary("User interrupted action")
                .urgency(NotificationUrgency::Low)
                .timeout(Timeout::Milliseconds(4000))
                .show().unwrap();
        },
        ErrorKind::InvalidInput => {
            Notification::new()
                .summary("Action failed!")
                .body("User input is invalid")
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
