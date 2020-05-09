use crate::commands::utils::choose_entry;

pub fn get(path: Option<&str>,
           id: Option<&str>) {

    let mut entry = choose_entry(path, id).expect("could not choose entry");
    entry.hidden = false;
    println!("{}", entry);

}
