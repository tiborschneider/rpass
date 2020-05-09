mod delete;
mod get;
mod insert;
mod interactive;
mod list;
mod utils;
mod mv;
mod passwd;
mod edit;
mod fix_index;

pub use delete::delete;
pub use get::get;
pub use insert::insert;
pub use interactive::interactive;
pub use list::list;
pub use mv::mv;
pub use passwd::passwd;
pub use edit::edit;
pub use fix_index::fix_index;

pub enum Error {
    InvalidInput,
    IOError(std::io::Error)
}
