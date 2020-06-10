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

use serde::Deserialize;
use toml;
use dirs::config_dir;
use lazy_static::lazy_static;

const CONFIG_FILE: &str = "rpass/config.toml";

lazy_static!{
    pub static ref CFG_STRING: String = {
        let mut config_file = config_dir().unwrap();
        config_file.push(CONFIG_FILE);
        if config_file.is_file() {
            std::fs::read_to_string(config_file).unwrap()
        } else {
            String::new()
        }
    };

    pub static ref CFG: Config<'static> = {
        toml::from_str::<ConfigBuilder<'static>>(&CFG_STRING).unwrap().build()
    };
}

#[derive(Debug, Deserialize)]
pub struct ConfigBuilder<'a> {
    #[serde(borrow)]
    pub main: Option<ConfigMainBuilder<'a>>,
    pub theme: Option<ConfigThemeBuilder<'a>>,
    pub pass: Option<ConfigPassBuilder<'a>>
}

#[derive(Debug, Deserialize)]
pub struct ConfigMainBuilder<'a> {
    pub uuid_folder: Option<&'a str>,
    pub index_entry: Option<&'a str>,
    pub index_file: Option<&'a str>,
    pub sync_folder: Option<&'a str>,
    pub sync_commit_file: Option<&'a str>,
    pub last_command_file: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigThemeBuilder<'a> {
    pub theme_name: Option<&'a str>,
    pub key_alpha: Option<&'a str>,
    pub link_color: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigPassBuilder<'a> {
    pub user_key: Option<&'a str>,
    pub user_key_alt: Option<&'a str>,
    pub uuid_key: Option<&'a str>,
    pub path_key: Option<&'a str>,
    pub url_key: Option<&'a str>,
}

impl<'a> ConfigBuilder<'a> {
    fn build(mut self) -> Config<'a> {
        Config {
            main: self.main.take().unwrap_or(ConfigMainBuilder::new()).build(),
            theme: self.theme.take().unwrap_or(ConfigThemeBuilder::new()).build(),
            pass: self.pass.take().unwrap_or(ConfigPassBuilder::new()).build()
        }
    }
}

impl<'a> ConfigMainBuilder<'a> {
    fn new() -> Self {
        Self {
            uuid_folder: None,
            index_entry: None,
            index_file: None,
            sync_folder: None,
            sync_commit_file: None,
            last_command_file: None,
        }
    }

    fn build(mut self) -> ConfigMain<'a> {
        ConfigMain {
            uuid_folder: self.uuid_folder.take().unwrap_or("uuids"),
            index_entry: self.index_entry.take().unwrap_or("index"),
            index_file: self.index_file.take().unwrap_or("index.gpg"),
            sync_folder: self.sync_folder.take().unwrap_or(".sync"),
            sync_commit_file: self.sync_commit_file.take().unwrap_or(".sync_commit"),
            last_command_file: self.last_command_file.take().unwrap_or(".cache/rpass_last"),
        }
    }
}

impl<'a> ConfigThemeBuilder<'a> {
    fn new() -> Self {
        Self {
            theme_name: None,
            key_alpha: None,
            link_color: None,
        }
    }

    fn build(mut self) -> ConfigTheme<'a> {
        ConfigTheme {
            theme_name: self.theme_name.take(),
            key_alpha: self.key_alpha.take().unwrap_or("50%"),
            link_color: self.link_color.take().unwrap_or("#7EAFE9"),
        }
    }
}

impl<'a> ConfigPassBuilder<'a> {
    fn new() -> Self {
        Self {
            user_key: None,
            user_key_alt: None,
            uuid_key: None,
            path_key: None,
            url_key: None,
        }
    }

    fn build(mut self) -> ConfigPass<'a> {
        ConfigPass {
            user_key: self.user_key.take().unwrap_or("user: "),
            user_key_alt: self.user_key_alt.take().unwrap_or("user: "),
            uuid_key: self.uuid_key.take().unwrap_or("uuid: "),
            path_key: self.path_key.take().unwrap_or("path: "),
            url_key: self.url_key.take().unwrap_or("url: "),
        }
    }
}


#[derive(Debug)]
pub struct Config<'a> {
    pub main: ConfigMain<'a>,
    pub theme: ConfigTheme<'a>,
    pub pass: ConfigPass<'a>
}

#[derive(Debug)]
pub struct ConfigMain<'a> {
    pub uuid_folder: &'a str,
    pub index_entry: &'a str,
    pub index_file: &'a str,
    pub sync_folder: &'a str,
    pub sync_commit_file: &'a str,
    pub last_command_file: &'a str,
}

#[derive(Debug)]
pub struct ConfigTheme<'a> {
    pub theme_name: Option<&'a str>,
    pub key_alpha: &'a str,
    pub link_color: &'a str,
}

#[derive(Debug)]
pub struct ConfigPass<'a> {
    pub user_key: &'a str,
    pub user_key_alt: &'a str,
    pub uuid_key: &'a str,
    pub path_key: &'a str,
    pub url_key: &'a str,
}
