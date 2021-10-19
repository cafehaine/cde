extern crate pest;
#[macro_use]
extern crate pest_derive;

use subprocess::{Exec, ExitStatus};
use swayipc::{reply::Output, Connection, Error as SwayError};

use cde_config::load_config;

mod kanshiconfig;
use kanshiconfig::{KanshiConfig, KanshiProfile};

fn fetch_screen_layout(conn: &mut Connection) -> Result<Vec<Output>, SwayError> {
    let outputs = conn.get_outputs()?;
    Result::Ok(outputs)
}

fn restart_kanshi() -> () {
    // TODO use varlink API directly?
    match Exec::shell("kanshictl reload").join() {
        Ok(ExitStatus::Exited(0)) => (),
        Ok(status) => println!("Could not restart kanshi: {:?}", status),
        Err(err) => println!("Couldn't restart kanshi: {:?}", err),
    }
}

fn main() {
    println!("Loading cde's configuration");
    let cde_config = load_config();

    println!("Starting the screen layout editor");
    match Exec::shell(cde_config.autokanshi.screen_layout_editor).join() {
        Ok(ExitStatus::Exited(0)) => (),
        Ok(status) => panic!("Screen layout editor exited with error: {:?}", status),
        Err(err) => panic!("Could not start screen layout editor: {:?}", err),
    }

    println!("Connecting to sway's IPC");
    let mut ipc_conn = match Connection::new() {
        Ok(value) => value,
        Err(err) => panic!("Cannot connect to sway's IPC: {:?}", err.name()),
    };

    println!("Fetching the current screen layout");
    let screen_layout = match fetch_screen_layout(&mut ipc_conn) {
        Ok(value) => value,
        Err(err) => panic!("Couldn't fetch the current screen layout: {:?}", err.name()),
    };

    println!("Loading the kanshi config");
    let mut config = KanshiConfig::load();

    println!("Detecting matching profile");
    let new_profile = KanshiProfile::from_layout(&screen_layout);
    let detected_profile = config.detect_profile(&screen_layout);
    match detected_profile {
        Some(profile_index) => {
            println!("Overriding previous profile");
            config.replace_profile(profile_index, new_profile);
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
