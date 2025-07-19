/*
Copyright (C) 2025 Niklas Liesch <niklas.liesch@protonmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtemisConfig {
    base_url: String,
}

impl Default for ArtemisConfig {
    fn default() -> Self {
        Self {
            base_url: "https://artemis-app.inf.tu-dresden.de".to_string(),
        }
    }
}

impl ArtemisConfig {
    pub fn load(path: Option<&Path>) -> Self {
        let mut home = env::home_dir().expect("cant get HOME directory");
        home.push(".config/artemis-cli/config.toml");
        let cfg_path = path.unwrap_or(&home);

        if let Some(parent) = cfg_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let mut cfg_file = match OpenOptions::new().read(true).open(cfg_path) {
            Ok(f) => f,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    warn!("config not found you might need to run 'artemis-cli config base-url [BASEURL]' first");
                    Self::default().save(Some(cfg_path));
                    warn!(
                        "using default options: {:?} run 'artemis-cli config base-url [BASEURL]' first",
                        Self::default()
                    );
                    OpenOptions::new().read(true).open(cfg_path).unwrap()
                }
                _ => panic!("{e}"),
            },
        };

        let mut buf = String::new();
        cfg_file.read_to_string(&mut buf).expect("cant read cfg file");

        toml::from_str::<ArtemisConfig>(&mut buf).expect("cant parse config")
    }

    pub fn save(&self, path: Option<&Path>) {
        let mut home = env::home_dir().expect("cant get HOME directory");
        home.push(".config/artemis-cli/config.toml");
        let cfg_path = path.unwrap_or(&home);

        if let Some(parent) = cfg_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let mut cfg_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.unwrap_or(&cfg_path))
            .expect("unable to open config file");

        let cfg_str = toml::to_string(self).expect("cant Serialize config");
        cfg_file.write_all(cfg_str.as_bytes()).expect("cant write to cfg file");
    }

    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }

    pub fn get_base_url(&self) -> &String {
        &self.base_url
    }
}
