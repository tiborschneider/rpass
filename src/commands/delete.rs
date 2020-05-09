use crate::commands::utils::{choose_entry, confirm};
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
