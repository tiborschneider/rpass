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

    let mut entry = choose_entry(path, id)?;

    if !use_rofi {
        println!("Cange password of {}", entry);
    }

    let passwd = match passwd {
        Some(x) => x,
        None => {
            match use_rofi {
                true => {
                    match question_rofi("password", None)? {
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
