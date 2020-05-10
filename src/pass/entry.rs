use std::fmt;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::iter;

use uuid::Uuid;

use crate::pass::index;

pub const ROOT_FOLDER: &str = "uuids";
const USER_KEY: &str = "user: ";
const PATH_KEY: &str = "path: ";
const URL_KEY: &str = "url: ";
const UUID_KEY: &str = "uuid: ";

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
    pub password: Option<String>,
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
        if let Some(ref password) = self.password {
            let hidden_pw: String = match self.hidden {
                true => iter::repeat("*").take(password.len()).collect(),
                false => password.clone()
            };
            write!(f, "    password: {}\n", hidden_pw)?;
        }
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
        if let Some(ref password) = self.password {
            let hidden_pw: String = match self.hidden {
                true => iter::repeat("*").take(password.len()).collect(),
                false => password.clone()
            };
            write!(f, "    password: {}\n", hidden_pw)?;
        }
        if let Some(ref path) = self.path {
            write!(f, "    path:     {}\n", path)?;
        }
        if let Some(ref url) = self.url {
            write!(f, "    url:      {}\n", url)?;
        }
        let mut lines_iter = self.raw.lines().into_iter();
        lines_iter.next();
        let mut raw_printed = false;
        for line in lines_iter {
            if !(line.starts_with(USER_KEY) ||
                    line.starts_with(PATH_KEY) ||
                    line.starts_with(UUID_KEY) ||
                    line.starts_with(URL_KEY) ||
                    line.len() == 0) {
                if !raw_printed {
                    write!(f, "    raw:\n")?;
                    raw_printed = true;
                }
                write!(f, "        {}\n", line)?;
            }
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

        let mut raw: String = String::new();

        // handle password
        raw.push_str(&password);

        // handle username
        if let Some(ref a_user) = username {
            raw.push_str("\n");
            raw.push_str(USER_KEY);
            raw.push_str(a_user);
        }

        // handle url
        if let Some(ref a_url) = url {
            raw.push_str("\n");
            raw.push_str(URL_KEY);
            raw.push_str(a_url);
        }

        // handle path
        raw.push_str("\n");
        raw.push_str(PATH_KEY);
        raw.push_str(&path);

        // generate uuid
        let id = Uuid::new_v4();
        let id_string = format!("{}", id);
        raw.push_str("\n");
        raw.push_str(UUID_KEY);
        raw.push_str(&id_string);

        Entry {
            username: username,
            password: Some(password),
            path: Some(path),
            url: url,
            uuid: id,
            raw: raw,
            hidden: true
        }

    }

    pub fn get(id: Uuid) -> Result<Entry, Error> {
        let mut raw = Entry::get_raw(id)?;
        if !raw.ends_with("\n") {
            raw.push('\n');
        }
        let mut e = Entry {
            username: None,
            password: None,
            path: None,
            url: None,
            uuid: id,
            raw: raw.clone(),
            hidden: true
        };

        // add password (first line)
        let mut lines = raw.lines();
        e.password = match lines.next() {
            Some(s) => Some(s.to_string()),
            None => None
        };

        // search for username and path
        for line in lines {
            if line.starts_with(USER_KEY) {
                e.username = Some(line[USER_KEY.len()..].to_string());
            }
            if line.starts_with(PATH_KEY) {
                e.path = Some(line[PATH_KEY.len()..].to_string());
            }
            if line.starts_with(URL_KEY) {
                e.url = Some(line[URL_KEY.len()..].to_string());
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

        let mut p = Command::new("pass")
            .arg("insert")
            .arg("--multiline")
            .arg(format!("{}/{}", ROOT_FOLDER, self.uuid))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;

        if let Some(mut writer) = p.stdin.take() {
            writer.write_all(self.raw[..].as_bytes())?;
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

    fn get_raw(id: Uuid) -> Result<String, Error> {

        let result_utf8 = Command::new("pass")
            .arg(format!("{}/{}", ROOT_FOLDER, id))
            .output()?
            .stdout;

        // parse to str
        match String::from_utf8(result_utf8) {
            Ok(r) => Ok(r),
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Cannot parse utf8!"))
        }

    }

    pub fn change_username(&mut self, username: Option<String>) -> Result<(), Error> {
        match (username, self.username.is_some()) {
            (Some(new_user), true) => { // replace username
                let raw_clone = self.raw.clone();
                self.raw = String::new();
                for line in raw_clone.lines() {
                    if line.starts_with(USER_KEY) {
                        self.raw.push_str(USER_KEY);
                        self.raw.push_str(new_user.as_ref());
                    } else {
                        self.raw.push_str(line);
                    }
                    self.raw.push('\n');
                }
                self.username = Some(new_user);
                self.write()
            },
            (None, true) => { // remove username
                let raw_clone = self.raw.clone();
                self.raw = String::new();
                for line in raw_clone.lines() {
                    if !line.starts_with(USER_KEY) {
                        self.raw.push_str(line);
                    }
                    self.raw.push('\n');
                }
                self.username = None;
                self.write()
            },
            (Some(new_user), false) => { // add username
                let raw_clone = self.raw.clone();
                self.raw = String::new();
                let lines: Vec<&str> = raw_clone.lines().collect();
                if lines.len() == 0 {
                    return Err(Error::new(ErrorKind::InvalidData, "No password found!"));
                }
                let mut lines_iter = lines.into_iter();
                // push password
                self.raw.push_str(lines_iter.next().unwrap());
                self.raw.push('\n');
                // push username
                self.raw.push_str(USER_KEY);
                self.raw.push_str(new_user.as_ref());
                self.raw.push('\n');
                // push the rest
                for line in lines_iter {
                    self.raw.push_str(line);
                    self.raw.push('\n');
                }
                self.username = Some(new_user);
                self.write()
            },
            (None, false) => { // do nothing
                Ok(())
            }
        }
    }

    pub fn change_url(&mut self, url: Option<String>) -> Result<(), Error> {
        match (url, self.url.is_some()) {
            (Some(new_url), true) => { // replace username
                let raw_clone = self.raw.clone();
                self.raw = String::new();
                for line in raw_clone.lines() {
                    if line.starts_with(URL_KEY) {
                        self.raw.push_str(URL_KEY);
                        self.raw.push_str(new_url.as_ref());
                    } else {
                        self.raw.push_str(line);
                    }
                    self.raw.push('\n');
                }
                self.url = Some(new_url);
                self.write()
            },
            (None, true) => { // remove username
                let raw_clone = self.raw.clone();
                self.raw = String::new();
                for line in raw_clone.lines() {
                    if !line.starts_with(URL_KEY) {
                        self.raw.push_str(line);
                    }
                    self.raw.push('\n');
                }
                self.url = None;
                self.write()
            },
            (Some(new_url), false) => { // add username
                let raw_clone = self.raw.clone();
                self.raw = String::new();
                for line in raw_clone.lines() {
                    self.raw.push_str(line);
                    self.raw.push('\n');
                }
                // push the url last
                self.raw.push_str(URL_KEY);
                self.raw.push_str(new_url.as_ref());
                self.raw.push('\n');
                self.url = Some(new_url);
                self.write()
            },
            (None, false) => { // do nothing
                Ok(())
            }
        }
    }

    pub fn change_password(&mut self, new_pw: String) -> Result<(), Error> {

        if self.password.is_none() {
            return Err(Error::new(ErrorKind::InvalidData, "No password present"));
        }

        // change the password
        let raw_clone = self.raw.clone();
        let mut lines_iter = raw_clone.lines();
        self.raw = String::new();

        // write the new password
        self.raw.push_str(format!("{}\n", new_pw).as_str());
        // skip first line, which must contain the password
        lines_iter.next();

        // write the rest
        for line in lines_iter {
            self.raw.push_str(line);
            self.raw.push('\n');
        }

        // update the password
        self.password = Some(new_pw);

        // write the changes
        self.write()

    }

    pub fn change_raw_line(&mut self, old_line: Option<String>, new_line: Option<String>) -> Result<(), Error> {

        match old_line {
            Some(old_line) => {

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
            },
            None => {
                // insert new line
                match new_line {
                    Some(new_line) => { self.raw.push_str(new_line.as_str());
                                        self.raw.push('\n');
                                        self.write() },
                    None => Ok(())
                }
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

        // change the raw content
        let old_raw = self.raw.clone();
        self.raw = String::new();
        let mut path_entered = false;
        for line in old_raw.lines() {
            if line.starts_with(PATH_KEY) {
                self.raw.push_str(PATH_KEY);
                self.raw.push_str(new_path.as_str());
                path_entered = true;
            } else {
                self.raw.push_str(line);
            }
            self.raw.push('\n');
        }

        if !path_entered {
            self.raw.push_str(PATH_KEY);
            self.raw.push_str(new_path.as_str());
            self.raw.push('\n');
        }

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

        if let Some(ref password) = self.password {
            let hidden_pw: String = match self.hidden {
                true => iter::repeat("*").take(password.len()).collect(),
                false => escape_pango(password.clone())
            };
            s.push_str(PANGO_PASSWORD_NAME);
            s.push_str(hidden_pw.as_ref());
            s.push('\n');
        } else {
            panic!("Password is required!")
        }

        s.push_str(PANGO_URL_NAME);
        match self.url.clone() {
            Some(url) => s.push_str(escape_pango(url).as_ref()),
            None => s.push_str(PANGO_EMPTY_NAME)
        };
        s.push('\n');

        let mut raw_str_printed = false;
        let mut lines_iter = self.raw.lines().into_iter();
        lines_iter.next();
        for line in lines_iter {
            if !(line.starts_with(USER_KEY) ||
                    line.starts_with(PATH_KEY) ||
                    line.starts_with(UUID_KEY) ||
                    line.starts_with(URL_KEY) ||
                    line.len() == 0) {
                if !raw_str_printed {
                    raw_str_printed = true;
                    s.push_str(PANGO_RAW_NAME);
                    s.push('\n');
                }
                s.push_str(escape_pango(line.to_string()).as_ref());
                s.push('\n');
            }
        }
        s
    }

}

fn escape_pango(s: String) -> String {
    s.replace("&", "&amp;").replace(">", "&gt;").replace("<", "&lt;")
}
