use std::io;
use std::io::prelude::*;                                                           
use text_io::read;

use crate::commands::utils::choose_entry;
use crate::pass;

pub fn delete(path: Option<&str>,
              id: Option<&str>,
              force: bool) {

    let entry = choose_entry(path, id).expect("cannot chose entry");

    if !force {
        println!("{}", entry);
        match confirm("Are you sure to delete this entry?") {
            true  => { pass::index::remove(entry.uuid).expect("Could not remove the entry!"); },
            false => { println!("Action aborted, nothing changed"); }
        }
    } else {
        pass::index::remove(entry.uuid).expect("Could not remove the entry!");
    }

}


fn confirm(q: &str) -> bool {
    print!("{} [y/N]: ", q);
    io::stdout().flush().ok().expect("Could not flush stdout");
    let answer: String = read!("{}\n");
    answer == "y" || answer == "Y"
}
