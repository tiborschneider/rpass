use std::io::prelude::*;                                                           
use std::io;

use rpassword;
use fake::{Fake, faker};
use text_io::read;

use crate::pass::entry::Entry;
use crate::commands::utils;

pub fn insert(path: Option<&str>,
              username: Option<&str>,
              password: Option<&str>,
              url: Option<&str>,
              generate: Option<usize>) {

    let path = match path {
        Some(s) => s.to_string(),
        None => match utils::gen_path_interactive() {
            Ok(s) => {
                println!("path: {}", s);
                s
            },
            Err(e) => panic!("Invalid path selected! {}", e)
        }
    };

    let username = match username {
        Some(s) => Some(s.to_string()),
        None => question("username")
    };

    let password = match generate {
        Some(x) => generate_password(x),
        None => match password {
            Some(s) => s.to_string(),
            None => {
                let mut passwd: String;
                loop {
                    passwd = rpassword::prompt_password_stdout("Enter a password: ").expect("Invalid password entered!");
                    let rp = rpassword::prompt_password_stdout("Repeat the password: ").expect("Invalid password entered!");
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
        None => question("url")
    };

    let e = Entry::new(username, password, url, path);
    println!("{}", e);

    e.write().expect("Cannot generate password!");
}

fn question(q: &str) -> Option<String> {
    print!("{}: ", q);
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    if answer.len() == 0 {
        None
    } else {
        Some(answer)
    }
}

fn generate_password(x: usize) -> String {
    let pw: String = faker::internet::en::Password(x..x+1).fake();
    println!("Password: {}", pw);
    pw
        
}
