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

use crate::config::CFG;
use lazy_static::lazy_static;
use rofi::pango::{FontSize, Pango};

lazy_static! {
    static ref SMALL_PANGO: Pango<'static> = {
        let mut p = Pango::with_capacity("", 2);
        p.size(FontSize::Small);
        p.alpha(CFG.theme.key_alpha);
        p
    };
    static ref BUTTON_PANGO: Pango<'static> = {
        let mut p = Pango::with_capacity("", 2);
        p.size(FontSize::Small);
        p.fg_color(CFG.theme.link_color);
        p
    };
    static ref BIG_BUTTON_PANGO: Pango<'static> = {
        let mut p = Pango::with_capacity("", 1);
        p.fg_color(CFG.theme.link_color);
        p
    };
}

pub const ROOT_FOLDER: &str = ".password-store";
pub const ENTRY_EXTENSION: &str = "gpg";
pub const GIT_FOLDER: &str = ".git";

pub const DISPLAY_PATH: &str = "path:   ";
pub const DISPLAY_UUID: &str = "uuid:   ";
pub const DISPLAY_USER: &str = "user:   ";
pub const DISPLAY_PASS: &str = "pass:   ";
pub const DISPLAY_URL: &str = "url:   ";
pub const DISPLAY_RAW: &str = "raw data ";

pub const DISPLAY_RAW_SEP: &str = ":   ";

pub const DISPLAY_EMPTY: &str = "empty";

pub const DISPLAY_BTN_SHOW_PWD: &str = "Show Password";
pub const DISPLAY_BTN_HIDE_PWD: &str = "Hide Password";
pub const DISPLAY_BTN_EDIT_ENTRY: &str = "Edit entry";
pub const DISPLAY_BTN_MAIN_MENU: &str = "Main menu";
pub const DISPLAY_BTN_NEW_RAW: &str = "New raw line";
pub const DISPLAY_BTN_DELETE: &str = "Delete";
pub const DISPLAY_BTN_CANCEL: &str = "Cancel";
pub const DISPLAY_BTN_NEW_PATH: &str = "New path";
pub const DISPLAY_BTN_CPY_USERNAME: &str = "Username";
pub const DISPLAY_BTN_CPY_PASSWORD: &str = "Password";
pub const DISPLAY_BTN_CPY_BOTH: &str = "Both";
pub const DISPLAY_BTN_EXIT: &str = "exit";

pub const DISPLAY_BTN_MM_GET: &str = "Get Entry";
pub const DISPLAY_BTN_MM_NEW: &str = "New Entry";
pub const DISPLAY_BTN_MM_EDIT: &str = "Edit Entry";
pub const DISPLAY_BTN_MM_EXIT: &str = "Exit";

pub fn format_small(s: &str) -> String {
    SMALL_PANGO.build_content(s)
}

pub fn format_button(s: &str) -> String {
    BUTTON_PANGO.build_content(s)
}

pub fn format_big_button(s: &str) -> String {
    BIG_BUTTON_PANGO.build_content(s)
}

/*
pub const PANGO_PATH_NAME: &str     = "<span size='small' alpha='50%'><tt>    path  </tt></span>";
pub const PANGO_UUID_NAME: &str     = "<span size='small' alpha='50%'><tt>    uuid  </tt></span>";
pub const PANGO_USERNAME_NAME: &str = "<span size='small' alpha='50%'><tt>username  </tt></span>";
pub const PANGO_PASSWORD_NAME: &str = "<span size='small' alpha='50%'><tt>password  </tt></span>";
pub const PANGO_URL_NAME: &str      = "<span size='small' alpha='50%'><tt>     url  </tt></span>";
pub const PANGO_RAW_NAME: &str      = "<span size='small' alpha='50%'><tt>raw data</tt></span>";
pub const PANGO_EMPTY_NAME: &str    = "<span size='small' alpha='50%'>empty</span>";

pub const PANGO_SHOW_PASSWORD_NAME: &str = "<span size='small' fgcolor='#7EAFE9'>Show Password</span>";
pub const PANGO_HIDE_PASSWORD_NAME: &str = "<span size='small' fgcolor='#7EAFE9'>Hide Password</span>";
pub const PANGO_EDIT_ENTRY_NAME: &str    = "<span size='small' fgcolor='#7EAFE9'>Edit entry</span>";
pub const PANGO_MAIN_MENU_NAME: &str     = "<span size='small' fgcolor='#7EAFE9'>Main menu</span>";
pub const PANGO_NEW_LINE_NAME: &str      = "<span size='small' fgcolor='#7EAFE9'>New raw line</span>";
pub const PANGO_DELETE_NAME: &str        = "<span size='small' fgcolor='#7EAFE9'>Delete</span>";
pub const PANGO_CANCEL_NAME: &str        = "<span size='small' fgcolor='#7EAFE9'>cancel</span>";
pub const PANGO_NEW_PATH_NAME: &str      = "<span size='small' fgcolor='#7EAFE9'>New path</span>";
pub const PANGO_COPY_USERNAME_NAME: &str = "<span size='small' fgcolor='#7EAFE9'>Username</span>";
pub const PANGO_COPY_PASSWORD_NAME: &str = "<span size='small' fgcolor='#7EAFE9'>Password</span>";
pub const PANGO_COPY_BOTH_NAME: &str     = "<span size='small' fgcolor='#7EAFE9'>Both</span>";
pub const PANGO_EXIT_NAME: &str          = "<span size='small' alpha='50%'>exit</span>";
*/
