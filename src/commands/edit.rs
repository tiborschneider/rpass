use crate::commands::utils::choose_entry;

pub fn edit(path: Option<&str>,
            id: Option<&str>) {

    choose_entry(path, id).expect("cannot choose entry!")
        .edit().expect("cannot edit the entry");

}
