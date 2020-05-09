use std::io::{Error, ErrorKind};

use crate::commands::utils::{choose_entry, confirm};
use crate::pass;

pub fn delete(path: Option<&str>,
              id: Option<&str>,
              force: bool,
              use_rofi: bool) -> Result<(), Error> {

    let entry = choose_entry(path, id)?;

    if !force {
        println!("{}", entry);
        match confirm("Are you sure to delete this entry?", use_rofi) {
            true  => pass::index::remove(entry.uuid),
            false => Err(Error::new(ErrorKind::Interrupted, "Not confirmed!"))
        }
    } else {
        pass::index::remove(entry.uuid)
    }

}
