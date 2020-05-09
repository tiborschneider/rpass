use std::io::Error;

use crate::commands::utils::choose_entry;

pub fn edit(path: Option<&str>,
            id: Option<&str>) -> Result<(), Error> {

    choose_entry(path, id)?.edit()

}
