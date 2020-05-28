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

pub const ROOT_FOLDER: &str = ".password-store";
pub const UUID_FOLDER: &str = "uuids";
pub const INDEX_ENTRY: &str = "index";
pub const INDEX_FILE: &str = "index.gpg";
pub const ENTRY_EXTENSION: &str = "gpg";
// pub const ENTRY_ENDING: &str = ".gpg";
pub const GIT_FOLDER: &str = ".git";
pub const SYNC_FOLDER: &str = ".sync";
pub const SYNC_COMMIT_FILE: &str = ".sync_commit";

pub const USER_KEY: &str     = "user: ";
pub const USER_KEY_ALT: &str = "username: ";
pub const PATH_KEY: &str     = "path: ";
pub const URL_KEY: &str      = "url: ";
pub const UUID_KEY: &str     = "uuid: ";

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

pub const LAST_COMMAND_FILE: &str = ".cache/rpass_last";
