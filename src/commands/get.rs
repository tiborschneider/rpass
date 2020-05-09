use std::io::Error;
use crate::commands::utils::choose_entry;

pub fn get(path: Option<&str>,
           id: Option<&str>) -> Result<(), Error> {

    let mut entry = choose_entry(path, id)?;
    entry.hidden = false;
    println!("{}", entry);
    Ok(())

}
