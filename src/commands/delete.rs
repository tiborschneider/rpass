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

use crate::errors::{Error, Result};
use crate::commands::utils::{choose_entry, confirm};
use crate::pass;

pub fn delete(path: Option<&str>,
              id: Option<&str>,
              force: bool,
              use_rofi: bool) -> Result<()> {

    let entry = choose_entry(path, id, use_rofi)?;

    if !force {
        if !use_rofi {
            println!("{}", entry);
        }
        match confirm("Are you sure to delete this entry?", use_rofi) {
            true  => pass::index::remove(entry.uuid),
            false => Err(Error::Interrupted)
        }
    } else {
        pass::index::remove(entry.uuid)
    }

}
