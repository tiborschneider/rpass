use std::clone::Clone;
use std::fmt::Display;
use std::io::{Error, ErrorKind};

use rustofi::components::{ActionList, ItemList, EntryBox};
use rustofi::window::Dimensions;
use rustofi::RustofiResult;

use crate::pass::index::{get_index, to_graph};

pub fn rofi_display_item<'a, T: Display + Clone>(rofi: &mut ItemList<'a, T>, prompt: String, lines: usize) -> RustofiResult {
    let extra = vec!["".to_string(), "[cancel]".to_string()];
    let mut display_options: Vec<String> = rofi.items.iter().map(|s| s.clone().to_string()).collect();
    let num_lines: i32 = if lines > display_options.len() {display_options.len() as i32} else {lines as i32};
    display_options = display_options.into_iter().chain(extra.clone()).collect();
    let response = rofi
        .window
        .clone()
        .lines(num_lines)
        .prompt(prompt)
        .add_args(vec!["-no-case-sensitive".to_string()])
        .show(display_options.clone());
    match response {
        Ok(input) => {
            if input == "[cancel]" || input == "" {
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
    let extra = vec!["".to_string(), "[cancel]".to_string()];
    let mut display_options: Vec<String> = rofi.actions.iter().map(|s| s.to_string()).collect();
    let num_lines: i32 = if lines > display_options.len() {display_options.len() as i32} else {lines as i32};
    display_options = display_options.into_iter().chain(extra.clone()).collect();
    let response = rofi
        .window
        .clone()
        .lines(num_lines)
        .prompt(prompt)
        .add_args(vec!["-no-case-sensitive".to_string()])
        .show(display_options.clone());
    match response {
        Ok(input) => {
            if input == "[cancel]" || input == "" {
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

pub fn gen_path_interactive() -> Result<String, Error> {
    match gen_path_recursive("".to_string()) {
        RustofiResult::Selection(s) => Ok(s),
        RustofiResult::Action(_)    => Err(Error::new(ErrorKind::Other, "Rofi returned an action instead of a selection!")),
        RustofiResult::Success      => Err(Error::new(ErrorKind::UnexpectedEof, "Success returned without a string!")),
        RustofiResult::Blank        => Err(Error::new(ErrorKind::Interrupted, "Blank option chosen!")),
        RustofiResult::Error        => Err(Error::new(ErrorKind::Other, "Rofi returned unexpected error")),
        RustofiResult::Cancel       => Err(Error::new(ErrorKind::Interrupted, "Cancel option chosen!")),
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
    next_nodes.push("[New]".to_string());
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
        "[New]" => ask_for_path(&cur_path),
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
