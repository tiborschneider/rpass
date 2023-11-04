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
use fake::{faker, Fake};

use crate::commands::{get, utils};
use crate::pass::entry::Entry;

pub fn insert(
    path: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    url: Option<&str>,
    generate: Option<usize>,
    use_rofi: bool,
) -> Result<()> {
    let path = match path {
        Some(s) => s.to_string(),
        None => match use_rofi {
            true => utils::gen_path_interactive()?,
            false => match utils::question("path", use_rofi)? {
                Some(s) => s,
                None => return Err(Error::InvalidInput("Path is required")),
            },
        },
    };

    let username = match username {
        Some(s) => Some(s.to_string()),
        None => utils::question("username", use_rofi)?,
    };

    let password = match generate {
        Some(x) => generate_password(x),
        None => match password {
            Some(s) => s.to_string(),
            None => {
                if use_rofi {
                    match utils::question("password", use_rofi)? {
                        Some(pw) => pw,
                        None => return Err(Error::InvalidInput("Password is required!")),
                    }
                } else {
                    let mut passwd: String;
                    loop {
                        passwd = rpassword::prompt_password_stdout("Enter a password: ")?;
                        let rp = rpassword::prompt_password_stdout("Repeat the password: ")?;
                        if passwd == rp {
                            break;
                        } else {
                            println!("The two passwords don't match. try again!");
                        }
                    }
                    passwd
                }
            }
        },
    };

    let url = match url {
        Some(s) => Some(s.to_string()),
        None => utils::question("url", use_rofi)?,
    };

    let e = Entry::new(username, password, url, path);

    e.create()?;

    if use_rofi {
        get(
            None,
            Some(format!("{}", e.uuid).as_str()),
            use_rofi,
            false,
            false,
        )
    } else {
        println!("Created {}", e);
        Ok(())
    }
}

fn generate_password(x: usize) -> String {
    let pw: String = faker::internet::en::Password(x..x + 1).fake();
    println!("Password: {}", pw);
    pw
}
