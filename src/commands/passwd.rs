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

use std::io::{Error, ErrorKind};

use rpassword;
use fake::{Fake, faker};

use crate::commands::utils::{choose_entry, question_rofi};

pub fn passwd(path: Option<&str>,
              id: Option<&str>,
              new_passwd: Option<&str>,
              generate: Option<usize>,
              use_rofi: bool) -> Result<(), Error> {

    let passwd = match generate {
        Some(x) => Some(faker::internet::en::Password(x..x+1).fake()),
        None => match new_passwd {
            Some(s) => Some(s.to_string()),
            None => None
        }
    };

    let mut entry = choose_entry(path, id, use_rofi)?;

    if !use_rofi {
        println!("Cange password of {}", entry);
    }

    let passwd = match passwd {
        Some(x) => x,
        None => {
            match use_rofi {
                true => {
                    match question_rofi("password")? {
                        Some(pw) => pw,
                        None     => return Err(Error::new(ErrorKind::InvalidInput, "Password cannot be empty")),
                    }
                },
                false => {
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
        }
    };

    match passwd.len() {
        0 => Err(Error::new(ErrorKind::InvalidInput, "Password cannot be empty!")),
        _ => entry.change_password(passwd)
    }
    
}
