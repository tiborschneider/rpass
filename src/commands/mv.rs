use crate::commands::utils::{choose_entry, gen_path_interactive};

pub fn mv(path: Option<&str>,
          id: Option<&str>,
          dst: Option<&str>) {

    let mut entry = choose_entry(path, id).expect("could not choose entry");

    println!("Moving {}", entry);
    let dst_string = match dst {
        Some(s) => s.to_string(),
        None => match gen_path_interactive() {
            Ok(s) => s,
            Err(e) => panic!("Error generating the path interactively: {}", e)
        }
    };

    // pass::index::mv(entry.uuid, dst_string.clone()).expect("Could not move the key!");
    entry.change_path(dst_string.clone()).expect("Could not move the key!");

    println!("Moved entry to {}", dst_string);
}
