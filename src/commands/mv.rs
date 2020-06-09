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
use crate::commands::utils::{choose_entry, gen_path_interactive, question};

pub fn mv(path: Option<&str>,
          id: Option<&str>,
          dst: Option<&str>,
          use_rofi: bool) -> Result<()> {

    let mut entry = choose_entry(path, id, use_rofi)?;

    if !use_rofi {
        println!("Moving {}", entry);
    }

    let dst_string = match dst {
        Some(s) => s.to_string(),
        None => {
            let result = match use_rofi {
                true => gen_path_interactive()?,
                false => question("path", use_rofi)?
            };
            match result {
                Some(s) => s,
                None => return Err(Error::InvalidInput("New path is required!"))
            }
        }
    };

    // pass::index::mv(entry.uuid, dst_string.clone()).expect("Could not move the key!");
    entry.change_path(dst_string.clone())?;

    if !use_rofi {
        println!("Moved entry to {}", dst_string);
    }

    Ok(())
}
