use std::fmt;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::iter;

use uuid::Uuid;

use crate::pass::index;

pub const ROOT_FOLDER: &str = "uuids";
const USER_KEY: &str = "user: ";
const USER_KEY_ALT: &str = "username: ";
pub const PATH_KEY: &str = "path: ";
const URL_KEY: &str = "url: ";
pub const UUID_KEY: &str = "uuid: ";

pub const PANGO_PATH_NAME: &str     = "<span size='smaller' alpha='50%'><tt>    path  </tt></span>";
pub const PANGO_UUID_NAME: &str     = "<span size='smaller' alpha='50%'><tt>    uuid  </tt></span>";
pub const PANGO_USERNAME_NAME: &str = "<span size='smaller' alpha='50%'><tt>username  </tt></span>";
pub const PANGO_PASSWORD_NAME: &str = "<span size='smaller' alpha='50%'><tt>password  </tt></span>";
pub const PANGO_URL_NAME: &str      = "<span size='smaller' alpha='50%'><tt>     url  </tt></span>";
pub const PANGO_RAW_NAME: &str      = "<span size='smaller' alpha='50%'><tt>raw data</tt></span>";
const PANGO_EMPTY_NAME: &str        = "<span size='smaller' alpha='50%'>empty</span>";

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
            username: username,
            password: password,
            path: Some(path),
            url: url,
            uuid: Uuid::new_v4(),
            raw: String::new(),
            hidden: true
        }
    }

    pub fn get(id: Uuid) -> Result<Entry, Error> {
        let mut e = Entry::from_path(format!("{}/{}", ROOT_FOLDER, id))?;
        if e.uuid != id {
            println!("[Warning] Fixing UUID stored in entry {}", id);
            e.uuid = id;
        }
        Ok(e)
    }

    pub fn from_path(path: String) -> Result<Entry, Error> {

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
                .arg(path.as_str())
                .output()?
                .stdout) {
            Ok(r) => r,
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Cannot parse utf8!"))
        };

        // parse the raw content, and remove the lines which are parsed (or add the lines which are not parsed to raw)
        let mut lines = raw.lines();
        e.password = match lines.next() {
            Some(s) => s.to_string(),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("Pass entry {} is empty!", path.as_str())))
        };

        // search for username and path
        for line in lines {
            let line_lower = line.to_lowercase();
            if line_lower.starts_with(USER_KEY) || line_lower.starts_with(USER_KEY_ALT) {
                e.username = Some(line[USER_KEY.len()..].to_string());
            } else if line_lower.starts_with(PATH_KEY) {
                e.path = Some(line[PATH_KEY.len()..].to_string());
            } else if line_lower.starts_with(URL_KEY) {
                e.url = Some(line[URL_KEY.len()..].to_string());
            } else if line_lower.starts_with(UUID_KEY) {
                e.uuid = match Uuid::parse_str(&line[UUID_KEY.len()..]) {
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
            raw_content.push_str(USER_KEY);
            raw_content.push_str(username);
            raw_content.push('\n');
        }
        // push url
        if let Some(ref url) = self.url {
            raw_content.push_str(URL_KEY);
            raw_content.push_str(url);
            raw_content.push('\n');
        }
        // push path
        if let Some(ref path) = self.path {
            raw_content.push_str(PATH_KEY);
            raw_content.push_str(path);
            raw_content.push('\n');
        }
        // push all the content of self.raw
        raw_content.push_str(self.raw.as_str());
        //push the uuid last
        raw_content.push_str(UUID_KEY);
        raw_content.push_str(format!("{}", self.uuid).as_ref());
        raw_content.push('\n');

        // write raw_content to pass
        let mut p = Command::new("pass")
            .arg("insert")
            .arg("--multiline")
            .arg(format!("{}/{}", ROOT_FOLDER, self.uuid))
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
            .arg(format!("{}/{}", ROOT_FOLDER, self.uuid))
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

        s.push_str(PANGO_PATH_NAME);
        s.push_str(escape_pango(self.path.clone().unwrap()).as_str());
        s.push('\n');

        s.push_str(PANGO_UUID_NAME);
        s.push_str(format!("{}", self.uuid).as_ref());
        s.push('\n');
        
        s.push_str(PANGO_USERNAME_NAME);
        match self.username.clone() {
            Some(username) => s.push_str(escape_pango(username).as_ref()),
            None => s.push_str(PANGO_EMPTY_NAME)
        };
        s.push('\n');

        let hidden_pw: String = match self.hidden {
            true => iter::repeat("*").take(self.password.len()).collect(),
            false => escape_pango(self.password.clone())
        };
        s.push_str(PANGO_PASSWORD_NAME);
        s.push_str(hidden_pw.as_ref());
        s.push('\n');

        s.push_str(PANGO_URL_NAME);
        match self.url.clone() {
            Some(url) => s.push_str(escape_pango(url).as_ref()),
            None => s.push_str(PANGO_EMPTY_NAME)
        };
        s.push('\n');

        let mut raw_str_printed = false;
        for line in self.raw.lines() {
            if !raw_str_printed {
                raw_str_printed = true;
                s.push_str(PANGO_RAW_NAME);
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
