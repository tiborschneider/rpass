use std::io::{Error, ErrorKind};
use rpassword;
use fake::{Fake, faker};

use crate::pass::entry::Entry;
use crate::commands::utils;

pub fn insert(path: Option<&str>,
              username: Option<&str>,
              password: Option<&str>,
              url: Option<&str>,
              generate: Option<usize>,
              use_rofi: bool) -> Result<(), Error> {

    let path = match path {
        Some(s) => s.to_string(),
        None => {
            if use_rofi {
                match utils::gen_path_interactive()? {
                    Some(s) => {
                        println!("path: {}", s);
                        s
                    },
                    None => return Err(Error::new(ErrorKind::Interrupted, "path is required!")),
                }
            } else {
                match utils::question("path", use_rofi) {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::Interrupted, "path is required!"))
                }
            }
        }
    };

    let username = match username {
        Some(s) => Some(s.to_string()),
        None => utils::question("username", use_rofi)
    };

    let password = match generate {
        Some(x) => generate_password(x),
        None => match password {
            Some(s) => s.to_string(),
            None => {
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
    };

    let url = match url {
        Some(s) => Some(s.to_string()),
        None => utils::question("url", use_rofi)
    };

    let e = Entry::new(username, password, url, path);
    println!("{}", e);

    e.create()
}

fn generate_password(x: usize) -> String {
    let pw: String = faker::internet::en::Password(x..x+1).fake();
    println!("Password: {}", pw);
    pw
        
}
