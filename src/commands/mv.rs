use std::io::{Error, ErrorKind};
use crate::commands::utils::{choose_entry, gen_path_interactive, question};

pub fn mv(path: Option<&str>,
          id: Option<&str>,
          dst: Option<&str>,
          use_rofi: bool) -> Result<(), Error> {

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
                None => return Err(Error::new(ErrorKind::Interrupted, "New path is required!"))
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
