use std::path::Path;

use subprocess::Exec;
use swayipc::{reply::Output, Connection, Error as SwayError};

struct AutokanshiConfig {
    screen_layout_editor: String,
}

impl AutokanshiConfig {
    fn load() -> AutokanshiConfig {
        println!("    TODO!");
        AutokanshiConfig {
            screen_layout_editor: String::from("wdisplays"),
        } //TODO
    }
}

struct KanshiDirective {
    words: Vec<String>,
}

struct KanshiProfile {
    name: Option<String>,
    directives: Vec<KanshiDirective>,
}

impl KanshiProfile {
    fn from_layout(screen_layout: &Vec<Output>) -> KanshiProfile {
        println!("    TODO!");
        KanshiProfile {
            name: None,
            directives: Vec::new(), //TODO
        }
    }
}

struct KanshiConfig<'a> {
    config_path: &'a Path,
    profiles: Vec<KanshiProfile>,
}

impl KanshiConfig<'_> {
    fn append_profile(&self, profile: KanshiProfile) -> () {
        println!("    TODO!");
        //TODO
    }

    fn detect_profile(&self, layout: &Vec<Output>) -> Option<&KanshiProfile> {
        println!("    TODO!");
        None //TODO
    }

    fn replace_profile(&self, to_replace: &KanshiProfile, profile: KanshiProfile) -> () {
        println!("    TODO!");
        //TODO
    }

    fn save(&self) -> () {
        println!("    TODO!");
        //TODO
    }

    fn load() -> KanshiConfig<'static> {
        println!("    TODO!");
        KanshiConfig {
            config_path: Path::new(""),
            profiles: Vec::new(),
        } // TODO
    }
}

fn fetch_screen_layout(conn: &mut Connection) -> Result<Vec<Output>, SwayError> {
    let outputs = conn.get_outputs()?;
    Result::Ok(outputs)
}

fn restart_kanshi() -> () {
    println!("    TODO!");
    //TODO
}

fn main() {
    println!("Loading autokanshi's configuration");
    let autokanshi_config = AutokanshiConfig::load();

    println!("Starting the screen layout editor");
    match Exec::shell(autokanshi_config.screen_layout_editor).join() {
        Ok(_) => {}
        Err(err) => {
            panic!("Could not start screen layout editor: {:?}", err)
        }
    }

    println!("Connecting to sway's IPC");
    let mut ipc_conn = match Connection::new() {
        Ok(value) => value,
        Err(err) => {
            panic!("Cannot connect to sway's IPC: {:?}", err.name())
        }
    };

    println!("Fetching the current screen layout");
    let screen_layout = match fetch_screen_layout(&mut ipc_conn) {
        Ok(value) => value,
        Err(err) => {
            panic!("Couldn't fetch the current screen layout: {:?}", err.name())
        }
    };

    println!("Loading the kanshi config");
    let config = KanshiConfig::load();

    println!("Detecting matching profile");
    let new_profile = KanshiProfile::from_layout(&screen_layout);
    match config.detect_profile(&screen_layout) {
        Some(profile) => {
            println!("Overriding previous profile");
            config.replace_profile(&profile, new_profile);
        }
        None => {
            println!("Creating new profile");
            config.append_profile(new_profile);
        }
    };

    println!("Saving kanshi config");
    config.save();

    println!("Restarting kanshi");
    restart_kanshi();

    println!("Done")
}
