use std::io::Error;

use rustofi::window::Window;

use crate::commands::utils::{choose_entry, copy_to_clipboard};
use crate::commands::edit;
use crate::pass::entry::Entry;

const SHOW_PASSWORD_NAME: &str = "[Show Password]";
const HIDE_PASSWORD_NAME: &str = "[Hide Password]";
const EDIT_ENTRY_NAME: &str = "[Edit entry]";
const MAIN_MENU_NAME: &str = "[Main menu]";

pub fn get(path: Option<&str>,
           id: Option<&str>,
           use_rofi: bool) -> Result<(), Error> {

    let mut entry = choose_entry(path, id)?;
    if use_rofi {
        get_rofi_menu(&mut entry)
    } else {
        entry.hidden = false;
        println!("{:?}", entry);
        Ok(())
    }

}

fn get_rofi_menu(entry: &mut Entry) -> Result<(), Error> {
    loop {
        let mut lines: Vec<String> = entry.get_string().lines().map(|x| x.to_string()).collect();
        lines.push(String::new());
        if entry.hidden {
            lines.push(SHOW_PASSWORD_NAME.to_string());
        } else {
            lines.push(HIDE_PASSWORD_NAME.to_string());
        }
        lines.push(EDIT_ENTRY_NAME.to_string());
        lines.push(MAIN_MENU_NAME.to_string());
        match Window::new("Entry").lines(lines.len() as i32).format('s').show(lines.clone()) {
            Ok(s) => {
                match get_menu_action(s) {
                    GetMenuAction::CopyPath     => copy_to_clipboard(entry.path.clone().unwrap(), "path", Some(5000))?,
                    GetMenuAction::CopyUuid     => copy_to_clipboard(format!("{}", entry.uuid), "UUID", Some(5000))?,
                    GetMenuAction::CopyUsername => copy_to_clipboard(entry.username.clone().unwrap(), "Username", Some(5000))?,
                    GetMenuAction::CopyPassword => copy_to_clipboard(entry.password.clone().unwrap(), "Password", Some(5000))?,
                    GetMenuAction::CopyUrl      => copy_to_clipboard(entry.url.clone().unwrap(), "URL", Some(5000))?,
                    GetMenuAction::CopyOther(s) => copy_to_clipboard(s, "Custom entry", Some(5000))?,
                    GetMenuAction::ShowPassword => entry.hidden = false,
                    GetMenuAction::HidePassword => entry.hidden = true,
                    GetMenuAction::EditEntry    => { edit(None, Some(format!("{}", entry.uuid).as_str()), true)?;
                                                     break; },
                    GetMenuAction::Exit         => break
                }
            }
            Err(_) => break
        }
    }
    Ok(())
}

enum GetMenuAction {
    CopyPath,
    CopyUuid,
    CopyUsername,
    CopyPassword,
    CopyUrl,
    CopyOther(String),
    ShowPassword,
    HidePassword,
    EditEntry,
    Exit
}

fn get_menu_action(s: String) -> GetMenuAction {
    if s == SHOW_PASSWORD_NAME { GetMenuAction::ShowPassword }
    else if s == HIDE_PASSWORD_NAME { GetMenuAction::HidePassword }
    else if s == EDIT_ENTRY_NAME { GetMenuAction::EditEntry }
    else if s.starts_with("path: ") { GetMenuAction::CopyPath }
    else if s.starts_with("uuid: ") { GetMenuAction::CopyUuid }
    else if s.starts_with("username: ") { GetMenuAction::CopyUsername }
    else if s.starts_with("password: ") { GetMenuAction::CopyPassword }
    else if s.starts_with("url: ") { GetMenuAction::CopyUrl }
    else if s.len() > 0 && s != MAIN_MENU_NAME {GetMenuAction::CopyOther(s.clone()) }
    else { GetMenuAction::Exit }
}
