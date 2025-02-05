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

use crate::errors::Result;
use dirs::config_dir;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "rpass/config.toml";

lazy_static! {
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
        toml::from_str::<ConfigBuilder<'static>>(&CFG_STRING)
            .unwrap()
            .build()
    };
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigBuilder<'a> {
    #[serde(borrow)]
    pub main: Option<ConfigMainBuilder<'a>>,
    pub theme: Option<ConfigThemeBuilder<'a>>,
    pub pass: Option<ConfigPassBuilder<'a>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigMainBuilder<'a> {
    pub uuid_folder: Option<&'a str>,
    pub index_entry: Option<&'a str>,
    pub index_file: Option<&'a str>,
    pub sync_folder: Option<&'a str>,
    pub sync_commit_file: Option<&'a str>,
    pub last_command_file: Option<&'a str>,
    pub history_file: Option<&'a str>,
    pub history_days: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigThemeBuilder<'a> {
    pub theme_name: Option<&'a str>,
    pub key_alpha: Option<&'a str>,
    pub link_color: Option<&'a str>,
    pub main_screen_width: Option<usize>,
    pub width: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
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
            main: self
                .main
                .take()
                .unwrap_or_else(ConfigMainBuilder::new)
                .build(),
            theme: self
                .theme
                .take()
                .unwrap_or_else(ConfigThemeBuilder::new)
                .build(),
            pass: self
                .pass
                .take()
                .unwrap_or_else(ConfigPassBuilder::new)
                .build(),
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
            history_file: None,
            history_days: None,
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
            history_file: self
                .last_command_file
                .take()
                .unwrap_or(".cache/rpass_history"),
            history_days: self.history_days.take().unwrap_or(50),
        }
    }
}

impl<'a> ConfigThemeBuilder<'a> {
    fn new() -> Self {
        Self {
            theme_name: None,
            key_alpha: None,
            link_color: None,
            main_screen_width: None,
            width: None,
        }
    }

    fn build(mut self) -> ConfigTheme<'a> {
        ConfigTheme {
            theme_name: self.theme_name.take(),
            key_alpha: self.key_alpha.take().unwrap_or("50%"),
            link_color: self.link_color.take().unwrap_or("#ffffff"),
            main_screen_width: self.main_screen_width.take().unwrap_or(400),
            width: self.width.take().unwrap_or(600),
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
    pub pass: ConfigPass<'a>,
}

#[derive(Debug)]
pub struct ConfigMain<'a> {
    pub uuid_folder: &'a str,
    pub index_entry: &'a str,
    pub index_file: &'a str,
    pub sync_folder: &'a str,
    pub sync_commit_file: &'a str,
    pub last_command_file: &'a str,
    pub history_file: &'a str,
    pub history_days: u64,
}

#[derive(Debug)]
pub struct ConfigTheme<'a> {
    pub theme_name: Option<&'a str>,
    pub key_alpha: &'a str,
    pub link_color: &'a str,
    pub main_screen_width: usize,
    pub width: usize,
}

#[derive(Debug)]
pub struct ConfigPass<'a> {
    pub user_key: &'a str,
    pub user_key_alt: &'a str,
    pub uuid_key: &'a str,
    pub path_key: &'a str,
    pub url_key: &'a str,
}

/// Store the config to file
pub fn store_config() -> Result<()> {
    let default_config = toml::from_str::<ConfigBuilder<'static>>("")
        .unwrap()
        .build();
    let write_config: ConfigBuilder = ConfigBuilder {
        main: Some(ConfigMainBuilder {
            uuid_folder: Some(default_config.main.uuid_folder),
            index_entry: Some(default_config.main.index_entry),
            index_file: Some(default_config.main.index_file),
            sync_folder: Some(default_config.main.sync_folder),
            sync_commit_file: Some(default_config.main.sync_commit_file),
            last_command_file: Some(default_config.main.last_command_file),
            history_file: Some(default_config.main.history_file),
            history_days: Some(default_config.main.history_days),
        }),
        theme: Some(ConfigThemeBuilder {
            theme_name: default_config.theme.theme_name,
            key_alpha: Some(default_config.theme.key_alpha),
            link_color: Some(default_config.theme.link_color),
            main_screen_width: Some(default_config.theme.main_screen_width),
            width: Some(default_config.theme.width),
        }),
        pass: Some(ConfigPassBuilder {
            user_key: Some(default_config.pass.user_key),
            user_key_alt: Some(default_config.pass.user_key_alt),
            uuid_key: Some(default_config.pass.uuid_key),
            path_key: Some(default_config.pass.path_key),
            url_key: Some(default_config.pass.url_key),
        }),
    };

    let config_str = toml::to_string_pretty(&write_config).unwrap();
    let mut config_file = config_dir().unwrap();
    config_file.push(CONFIG_FILE);
    std::fs::create_dir_all(config_file.parent().unwrap())?;
    std::fs::write(config_file, config_str)?;

    Ok(())
}
