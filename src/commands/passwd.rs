use rpassword;
use fake::{Fake, faker};

use crate::commands::utils::choose_entry;

pub fn passwd(path: Option<&str>,
              id: Option<&str>,
              new_passwd: Option<&str>,
              generate: Option<usize>) {

    let passwd = match generate {
        Some(x) => Some(faker::internet::en::Password(x..x+1).fake()),
        None => match new_passwd {
            Some(s) => Some(s.to_string()),
            None => None
        }
    };

    let mut entry = choose_entry(path, id).expect("could not choose entry");

    println!("Cange password of {}", entry);

    let passwd = match passwd {
        Some(x) => x,
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
    };

    entry.change_password(if passwd.len() > 0 { Some(passwd) } else { None }).expect("Cannot change password");
    
}
