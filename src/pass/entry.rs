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

use std::fmt;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::iter;

use uuid::Uuid;

use crate::pass::index;
use crate::def;
use crate::config::CFG;

#[derive(Clone)]
pub struct Entry {
    pub username: Option<String>,
    pub password: String,
    pub path: Option<String>,
    pub url: Option<String>,
    pub uuid: Uuid,
    pub raw: String,
    pub hidden: bool
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        write!(f, "Entry: {}\n", self.uuid)?;
        if let Some(ref username) = self.username {
            write!(f, "    username: {}\n", username)?;
        }
        let hidden_pw: String = match self.hidden {
            true => iter::repeat("*").take(self.password.len()).collect(),
            false => self.password.clone()
        };
        write!(f, "    password: {}\n", hidden_pw)?;
        if let Some(ref path) = self.path {
            write!(f, "    path:     {}\n", path)?;
        }
        if let Some(ref url) = self.url {
            write!(f, "    url:      {}\n", url)?;
        }
        Ok(())

    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        write!(f, "Entry: {}\n", self.uuid)?;
        if let Some(ref username) = self.username {
            write!(f, "    username: {}\n", username)?;
        }
        let hidden_pw: String = match self.hidden {
            true => iter::repeat("*").take(self.password.len()).collect(),
            false => self.password.clone()
        };
        write!(f, "    password: {}\n", hidden_pw)?;
        if let Some(ref path) = self.path {
            write!(f, "    path:     {}\n", path)?;
        }
        if let Some(ref url) = self.url {
            write!(f, "    url:      {}\n", url)?;
        }
        let mut raw_printed = false;
        for line in self.raw.lines() {
            if !raw_printed {
                write!(f, "    raw:\n")?;
                raw_printed = true;
            }
            write!(f, "        {}\n", line)?;
        }
        Ok(())

    }
}

#[allow(dead_code)]
impl Entry {

    pub fn new(username: Option<String>,
               password: String,
               url: Option<String>,
               path: String) -> Entry {

        Entry {
            username,
            password,
            path: Some(path),
            url,
            uuid: Uuid::new_v4(),
            raw: String::new(),
            hidden: true
        }
    }

    pub fn get(id: Uuid) -> Result<Entry, Error> {
        let mut e = Entry::from_path(format!("{}/{}", CFG.main.uuid_folder, id))?;
        if e.uuid != id {
            println!("[Warning] Fixing UUID stored in entry {}", id);
            e.uuid = id;
        }
        Ok(e)
    }

    pub fn from_path<S>(path: S) -> Result<Entry, Error>
    where
         S: AsRef<str>
    {

        let mut e = Entry {
            username: None,
            password: String::new(),
            path: None,
            url: None,
            uuid: Uuid::nil(),
            raw: String::new(),
            hidden: true
        };

        let raw = match String::from_utf8(
            Command::new("pass")
                .arg(path.as_ref())
                .output()?
                .stdout) {
            Ok(r) => r,
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Cannot parse utf8!"))
        };

        // parse the raw content, and remove the lines which are parsed (or add the lines which are not parsed to raw)
        let mut lines = raw.lines();
        e.password = match lines.next() {
            Some(s) => s.to_string(),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("Pass entry {} is empty!", path.as_ref())))
        };

        // search for username and path
        for line in lines {
            let line_lower = line.to_lowercase();
            if line_lower.starts_with(CFG.pass.user_key) {
                e.username = Some(line[CFG.pass.user_key.len()..].to_string());
            } else if line_lower.starts_with(CFG.pass.user_key_alt) {
                e.username = Some(line[CFG.pass.user_key_alt.len()..].to_string());
            } else if line_lower.starts_with(CFG.pass.path_key) {
                e.path = Some(line[CFG.pass.path_key.len()..].to_string());
            } else if line_lower.starts_with(CFG.pass.url_key) {
                e.url = Some(line[CFG.pass.url_key.len()..].to_string());
            } else if line_lower.starts_with(CFG.pass.uuid_key) {
                e.uuid = match Uuid::parse_str(&line[CFG.pass.uuid_key.len()..]) {
                    Ok(id) => id,
                    Err(_) => Uuid::nil()
                }
            } else {
                // line is not recognized! add line to raw
                if line.len() > 0 {
                    e.raw.push_str(line);
                    e.raw.push('\n');
                }
            }
        }

        Ok(e)
    }

    pub fn create(&self) -> Result<(), Error> {

        if self.path.is_none() {
            return Err(Error::new(ErrorKind::NotFound, "Entry has no path!"))
        }

        self.write()?;
        index::insert(self.uuid, self.path.as_ref().unwrap())?;
        Ok(())
    }

    pub fn write(&self) -> Result<(), Error> {

        // rebuild raw
        let mut raw_content: String = String::new();

        // push password
        raw_content.push_str(self.password.as_str());
        raw_content.push('\n');
        // push Username
        if let Some(ref username) = self.username {
            raw_content.push_str(CFG.pass.user_key);
            raw_content.push_str(username);
            raw_content.push('\n');
        }
        // push url
        if let Some(ref url) = self.url {
            raw_content.push_str(CFG.pass.url_key);
            raw_content.push_str(url);
            raw_content.push('\n');
        }
        // push path
        if let Some(ref path) = self.path {
            raw_content.push_str(CFG.pass.path_key);
            raw_content.push_str(path);
            raw_content.push('\n');
        }
        // push all the content of self.raw
        raw_content.push_str(self.raw.as_str());
        //push the uuid last
        raw_content.push_str(CFG.pass.uuid_key);
        raw_content.push_str(format!("{}", self.uuid).as_ref());
        raw_content.push('\n');

        // write raw_content to pass
        let mut p = Command::new("pass")
            .arg("insert")
            .arg("--multiline")
            .arg(format!("{}/{}", CFG.main.uuid_folder, self.uuid))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;

        if let Some(mut writer) = p.stdin.take() {
            writer.write_all(raw_content[..].as_bytes())?;
        }

        p.wait()?;

        Ok(())
    }

    pub fn edit(&mut self) -> Result<(), Error> {

        Command::new("pass")
            .arg("edit")
            .arg(format!("{}/{}", CFG.main.uuid_folder, self.uuid))
            .spawn()?.wait()?;

        // update the own settings and check if the path is unchanged. If not, update the path
        let old_path = self.path.clone().unwrap();

        let new_entry = Entry::get(self.uuid)?;
        self.username = new_entry.username.clone();
        self.password = new_entry.password.clone();
        self.path = new_entry.path.clone();
        self.url = new_entry.url.clone();
        self.raw = new_entry.raw.clone();

        let new_path = self.path.clone().unwrap();
        if old_path != new_path {
            println!("Path changed!, updating index file...");
            index::mv(self.uuid, new_path)?
        }

        Ok(())

    }

    pub fn change_username(&mut self, username: Option<String>) -> Result<(), Error> {
        if username.is_some() {
            self.username = username;
            self.write()
        } else {
            Ok(())
        }
    }

    pub fn change_url(&mut self, url: Option<String>) -> Result<(), Error> {
        if url.is_some() {
            self.url = url;
            self.write()
        } else {
            Ok(())
        }
    }

    pub fn change_password(&mut self, new_pw: String) -> Result<(), Error> {
        self.password = new_pw;
        self.write()
    }

    pub fn change_raw_line(&mut self, old_line: Option<String>, new_line: Option<String>) -> Result<(), Error> {

        if let Some(old_line) = old_line {

            // replace the old line
            let mut found: bool = false;
            let raw_clone = self.raw.clone();
            self.raw = String::new();

            for line in raw_clone.lines() {
                if line == old_line {
                    found = true;
                    match new_line.clone() {
                        Some(new_line) => { self.raw.push_str(new_line.as_str());
                                            self.raw.push('\n'); },
                        None => {}
                    }
                } else {
                    self.raw.push_str(line);
                    self.raw.push('\n');
                }
            }

            match found {
                true => self.write(),
                false => Err(Error::new(ErrorKind::InvalidInput, "Could not find the line to edit"))
            }

        } else {

            // insert new line
            match new_line {
                Some(new_line) => { self.raw.push_str(new_line.as_str());
                                    self.raw.push('\n');
                                    self.write() },
                None => Ok(())
            }
        }
    }

    pub fn change_path(&mut self, new_path: String) -> Result<(), Error> {
        self.change_path_keep_index(new_path.clone())?;

        // change index file
        index::mv(self.uuid, new_path)
    }

    pub fn change_path_keep_index(&mut self, new_path: String) -> Result<(), Error> {

        // set the new path
        self.path = Some(new_path.clone());
        self.write()

    }

    pub fn get_string(&self) -> String {

        let mut s = String::new();

        s.push_str(def::PANGO_PATH_NAME);
        s.push_str(escape_pango(self.path.clone().unwrap()).as_str());
        s.push('\n');

        s.push_str(def::PANGO_UUID_NAME);
        s.push_str(format!("{}", self.uuid).as_ref());
        s.push('\n');
        
        s.push_str(def::PANGO_USERNAME_NAME);
        match self.username.clone() {
            Some(username) => s.push_str(escape_pango(username).as_ref()),
            None => s.push_str(def::PANGO_EMPTY_NAME)
        };
        s.push('\n');

        let hidden_pw: String = match self.hidden {
            true => iter::repeat("*").take(self.password.len()).collect(),
            false => escape_pango(self.password.clone())
        };
        s.push_str(def::PANGO_PASSWORD_NAME);
        s.push_str(hidden_pw.as_ref());
        s.push('\n');

        s.push_str(def::PANGO_URL_NAME);
        match self.url.clone() {
            Some(url) => s.push_str(escape_pango(url).as_ref()),
            None => s.push_str(def::PANGO_EMPTY_NAME)
        };
        s.push('\n');

        let mut raw_str_printed = false;
        for line in self.raw.lines() {
            if !raw_str_printed {
                raw_str_printed = true;
                s.push_str(def::PANGO_RAW_NAME);
                s.push('\n');
            }
            s.push_str(escape_pango(line.to_string()).as_ref());
            s.push('\n');
        }
        s
    }

}

fn escape_pango(s: String) -> String {
    s.replace("&", "&amp;").replace(">", "&gt;").replace("<", "&lt;")
}
