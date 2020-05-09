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
        if !self.hidden {
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
        let raw = Entry::get_raw(id)?;
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

    pub fn write(&self) -> Result<(), Error> {

        if self.path.is_none() {
            return Err(Error::new(ErrorKind::NotFound, "Entry has no path!"))
        }

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

        index::insert(self.uuid, self.path.as_ref().unwrap())?;

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

}
