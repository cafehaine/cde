use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
extern crate xdg;

#[derive(Deserialize, Debug, Serialize)]
pub struct CDEConfig {
    #[serde(default)]
    pub autokanshi: AutoKanshiConfig,
}
impl Default for CDEConfig {
    fn default() -> Self {
        Self {
            autokanshi: AutoKanshiConfig::default(),
        }
    }
}

fn default_screen_layout_editor() -> String {
    "wdisplays".to_string()
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AutoKanshiConfig {
    #[serde(default = "default_screen_layout_editor")]
    pub screen_layout_editor: String,
}
impl Default for AutoKanshiConfig {
    fn default() -> Self {
        Self {
            screen_layout_editor: default_screen_layout_editor(),
        }
    }
}

fn _load_config() -> Option<CDEConfig> {
    let xdg_dirs = match xdg::BaseDirectories::with_prefix("cde") {
        Ok(dirs) => dirs,
        Err(err) => {
            println!("Could not load config: {}.", err.to_string());
            return None;
        }
    };
    let file_path = match xdg_dirs.find_config_file("cde.toml") {
        Some(path) => {
            println!("Loading config from {:?}.", path);
            path
        }
        None => {
            println!("Could not load config: file not found.");
            return None;
        }
    };
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            println!("Could not load config: {}.", err.to_string());
            return None;
        }
    };
    let mut file_contents = String::new();
    match file.read_to_string(&mut file_contents) {
        Ok(_) => (),
        Err(err) => {
            println!("Could not load config: {}.", err.to_string());
            return None;
        }
    };
    match toml::from_str(&file_contents) {
        Ok(config) => Some(config),
        Err(err) => {
            println!("Could not load config: {}.", err.to_string());
            None
        }
    }
}

pub fn load_config() -> CDEConfig {
    match _load_config() {
        Some(config) => config,
        None => {
            println!("Could not load config file, using defaults.");
            CDEConfig::default()
        }
    }
}
