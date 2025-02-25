// rpass: a password manager based on pass, written in rust
// Copyright (C) 2020, Tibor Schneider
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see http://www.gnu.org/licenses/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot parse Utf8: {0}")]
    ParseUtf8(#[from] std::string::FromUtf8Error),
    #[error("Cannot parse Utf8: {0}")]
    ParseUtf8Str(#[from] std::str::Utf8Error),
    #[error("User interruption!")]
    Interrupted,
    #[error("Blank option chosen!")]
    Blank,
    #[error("Invalid Input: {0}")]
    InvalidInput(&'static str),
    #[error("Index file does not contain the path: {0}")]
    UnknownPath(String),
    #[error("Index file was not found!")]
    NoIndexFile,
    #[error("Managed folder (uuids) was not found!")]
    ManagedFolderNotFound,
    #[error("Could not modify entry raw line: {0}")]
    EntryRawEdit(String),
    #[error("Entry does not have a path: {0}")]
    EntryWithoutPath(String),
    #[error("Sync Error: {0}!")]
    Sync(&'static str),
    #[error("Empty entry found: {0}")]
    EmptyEntry(String),
    #[error("Cannot create clipboard context")]
    Clipboard,
    #[error("UUID Error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("Notification Error: {0}")]
    Notification(#[from] notify_rust::error::Error),
    #[error("Could not parse diff: {0}")]
    Unidiff(#[from] unidiff::Error),
    #[error("Rofi Error: {0}")]
    Rofi(rofi::Error),
    #[error("Could not create the XDo instance! {0}")]
    XDoCreation(#[from] libxdo::CreationError),
    #[error("XDo Error {0}")]
    XDo(#[from] libxdo::OpError),
    #[error("{0}")]
    Other(String),
}

impl From<rofi::Error> for Error {
    fn from(e: rofi::Error) -> Self {
        match e {
            rofi::Error::IoError(e) => Error::Rofi(rofi::Error::IoError(e)),
            rofi::Error::ParseIntError(e) => Error::Rofi(rofi::Error::ParseIntError(e)),
            rofi::Error::Interrupted => Error::Interrupted,
            rofi::Error::Blank => Error::Blank,
            rofi::Error::NotFound => Error::Rofi(rofi::Error::NotFound),
            rofi::Error::InvalidWidth(e) => Error::Rofi(rofi::Error::InvalidWidth(e)),
            rofi::Error::ConfigErrorMessageAndOptions => {
                Error::Rofi(rofi::Error::ConfigErrorMessageAndOptions)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
