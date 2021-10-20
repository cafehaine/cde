use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::string::ToString;

use chrono::{Datelike, Timelike, Utc};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use swayipc::reply::Output;
extern crate xdg;

#[derive(Parser)]
#[grammar = "kanshiconfig.pest"]
struct KanshiConfigParser;

#[derive(Eq, PartialEq)]
struct KanshiDirective {
    words: Vec<String>,
}

impl ToString for KanshiDirective {
    fn to_string(&self) -> String {
        self.words.join(" ")
    }
}

fn output_id(output: &Output) -> String {
    format!("\"{} {} {}\"", output.make, output.model, output.serial)
}

impl KanshiDirective {
    fn from_output(output: &Output) -> Self {
        let mut words = vec![String::from("output")];

        // Output id
        words.push(output_id(output));

        // Enable/disable
        words.push(match output.active {
            true => String::from("enable"),
            false => String::from("disable"),
        });

        // Mode
        match &output.current_mode {
            Some(mode) => {
                words.push(String::from("mode"));
                words.push(format!(
                    "{}x{}@{}Hz",
                    mode.width,
                    mode.height,
                    mode.refresh / 1000
                ));
            }
            None => (),
        }

        // Position
        words.push(String::from("position"));
        words.push(format!("{},{}", output.rect.x, output.rect.y));

        // Scale
        match &output.scale {
            Some(factor) => {
                words.push(String::from("scale"));
                words.push(factor.to_string());
            }
            None => (),
        }

        // Transform
        match &output.transform {
            Some(transform) => {
                words.push(String::from("transform"));
                words.push(transform.to_string());
            }
            None => (),
        }

        Self { words }
    }
}

#[derive(Eq, PartialEq)]
pub struct KanshiProfile {
    name: Option<String>,
    directives: Vec<KanshiDirective>,
}

impl ToString for KanshiProfile {
    fn to_string(&self) -> String {
        let str_directives = self
            .directives
            .iter()
            .map(|d| format!("    {}", d.to_string()));
        let config = str_directives.collect::<Vec<String>>().join("\n");
        match &(self.name) {
            None => String::from("profile {\n") + &config + &String::from("\n}"),
            Some(name) => {
                String::from("profile ")
                    + &name
                    + &String::from(" {\n")
                    + &config
                    + &String::from("\n}")
            }
        }
    }
}

impl KanshiProfile {
    pub fn from_layout(screen_layout: &[Output]) -> KanshiProfile {
        let directives = screen_layout
            .iter()
            .map(|o| KanshiDirective::from_output(o))
            .collect();
        let now = Utc::now();
        let name = format!(
            "autokanshi_{}_{}_{}__{}_{}_{}",
            now.day(),
            now.month(),
            now.year(),
            now.hour(),
            now.minute(),
            now.second()
        );
        KanshiProfile {
            name: Some(name),
            directives,
        }
    }

    pub fn get_screens(&self) -> HashSet<String> {
        let mut output = HashSet::new();
        for directive in &self.directives {
            if directive.words[0] == "output" {
                output.insert(directive.words[1].clone());
            }
        }
        output
    }
}

pub struct KanshiConfig {
    config_path: Box<PathBuf>,
    profiles: Vec<KanshiProfile>,
}

fn _build_directive(pair: Pair<Rule>) -> KanshiDirective {
    let words = pair.into_inner().map(|p| p.as_str().to_string()).collect();
    KanshiDirective { words }
}

fn _build_profile(pair: Pair<Rule>) -> KanshiProfile {
    let mut profile = KanshiProfile {
        name: None,
        directives: Vec::new(),
    };
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::word => {
                profile.name = Some(inner_pair.as_str().to_string());
            }
            Rule::directive => profile.directives.push(_build_directive(inner_pair)),
            Rule::EOI
            | Rule::WHITESPACE
            | Rule::NEWLINE
            | Rule::directive_start
            | Rule::profile
            | Rule::file => unreachable!(),
        }
    }
    profile
}

fn _build_config_from_tokens(pairs: Pairs<Rule>) -> Vec<KanshiProfile> {
    let mut profiles = Vec::new();
    for profile_pair in pairs {
        match profile_pair.as_rule() {
            Rule::profile => profiles.push(_build_profile(profile_pair)),
            Rule::EOI => (),
            Rule::WHITESPACE
            | Rule::NEWLINE
            | Rule::directive_start
            | Rule::file
            | Rule::word
            | Rule::directive => unreachable!(),
        }
    }
    profiles
}

fn _load_kanshi_config(xdg_dirs: &xdg::BaseDirectories) -> Option<KanshiConfig> {
    let file_path = match xdg_dirs.find_config_file("config") {
        Some(path) => {
            println!("Loading config from {:?}.", path);
            path
        }
        None => {
            println!("Could not load config: file not found.");
            return None;
        }
    };
    let mut file = match File::open(&file_path) {
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
    match KanshiConfigParser::parse(Rule::file, &file_contents) {
        Ok(pairs) => Some(KanshiConfig {
            config_path: Box::new(file_path),
            profiles: _build_config_from_tokens(pairs),
        }),
        Err(err) => panic!("Couldn't parse kanshi config file: {:?}", err),
    }
}

impl KanshiConfig {
    pub fn append_profile(&mut self, profile: KanshiProfile) {
        self.profiles.push(profile);
    }

    pub fn detect_profile(&self, layout: &[Output]) -> Option<usize> {
        let mut current_set = HashSet::new();

        for output in layout.iter() {
            current_set.insert(output_id(output));
        }

        for (index, profile) in self.profiles.iter().enumerate() {
            if profile.get_screens() == current_set {
                return Some(index);
            }
        }
        None
    }

    pub fn replace_profile(&mut self, to_replace: usize, mut profile: KanshiProfile) {
        // Not really optimised but not performance critical
        // The index will no longer be relevent
        profile.name = self.profiles[to_replace].name.clone();
        //TODO also copy 'exec' directives?
        self.profiles.remove(to_replace);
        self.profiles.push(profile);
    }

    pub fn save(&self) {
        let str_profiles = self.profiles.iter().map(|p| p.to_string());
        let mut parts: Vec<String> = vec![
            String::from("# GENERATED BY AUTOKANSHI, DON'T TRY TO EDIT MATCHING RULES BY HAND"),
            String::from(
                "# COMMENTS WILL BE REMOVED (however exec directives and profiles names are kept)",
            ),
        ];
        parts.extend(str_profiles);
        let config = parts.join("\n");
        let mut output = match File::create(*self.config_path.to_owned()) {
            Ok(file) => file,
            Err(err) => panic!("Cannot open kanshi config file for writing: {}.", err),
        };
        match output.write_all(config.as_bytes()) {
            Ok(_) => (),
            Err(err) => panic!("Cannot save kanshi config: {}.", err),
        }
    }

    pub fn load() -> Self {
        let xdg_dirs = match xdg::BaseDirectories::with_prefix("kanshi") {
            Ok(dirs) => dirs,
            Err(err) => panic!(
                "Could not fetch xdg base directory for config: {}.",
                err.to_string()
            ),
        };
        match _load_kanshi_config(&xdg_dirs) {
            Some(config) => config,
            None => match xdg_dirs.place_config_file("config") {
                Ok(path_buf) => Self {
                    config_path: Box::new(path_buf),
                    profiles: Vec::new(),
                },
                Err(err) => panic!(
                    "Could not create config directory for kanshi: {}.",
                    err.to_string()
                ),
            },
        }
    }
}
