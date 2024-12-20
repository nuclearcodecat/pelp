use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use clap::Parser;

const CONF_NAME: &str = "pelp.toml";

fn main() -> Result<(), Box<dyn Error>> {
    // clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!(r"
    ░▒▓███████▓▒░░▒▓████████▓▒░▒▓█▓▒░      ░▒▓███████▓▒░  
    ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░ 
    ░▒▓███████▓▒░░▒▓██████▓▒░ ░▒▓█▓▒░      ░▒▓███████▓▒░  
    ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░        
    ░▒▓█▓▒░      ░▒▓████████▓▒░▒▓████████▓▒░▒▓█▓▒░        
    ");

    let args = Args::parse();

    let user = match env::var("SUDO_HOME") {
        Ok(x) => {
            x
        },
        Err(e) => {
            eprintln!("PELP failed to get $SUDO_HOME, are you running as sudo? ({e})");
            println!("PELP retrying with $HOME");
            env::var("HOME")?
        }
    };
    let config_path = Path::new(&user).join(".config").join(CONF_NAME);

    let profile = get_profile(args.profile.unwrap(), config_path);    

    println!("PELP attempting to open {}...", args.device.clone().unwrap());

    // open device
    let file = match File::open(args.device.unwrap()) {
        Ok(x) => {
            println!("PELP opened device successfully!");
            x
        },
        Err(e) => {
            println!("PELP failed to open device");
            return Err(Box::new(e));
        }
    };
    let reader = BufReader::new(file);

    // line processing
    for line in reader.lines() {
        let line = line?;
        let line = line.trim_end();
        if line.as_bytes().is_empty() { continue };

        for entry in profile.iter() {
            if entry.ignore { continue };
            let trigger = entry.trigger.as_str();
            let was_triggered ;
            match entry.r#where.as_str() {
                "start of line" | "sol" => {
                    was_triggered = line.starts_with(trigger);
                },
                _ => {
                    was_triggered = line.contains(trigger);
                }
            }

            if was_triggered {
                if (entry.replace) {
                    line.replace(entry.trigger, &entry.replace_with);
                }
                println!("\x1b[{}m{}\x1b[0m", entry.color, line);
            } else {
                println!("{}", line);
            }            
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, value_name = "PROFILE NAME", default_value = "default", help = format!("name of the profile in ~/{}", CONF_NAME))]
    profile: Option<String>,
    #[arg(short, help = "name of the device to open (e.g. /dev/ttyUSB0)")]
    device: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ColorEntry {
    color: String,
    trigger: String,
    r#where: String,
    replace: bool,
    replace_with: String,
    ignore: bool,
}

type Profile = Vec<ColorEntry>; 

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Config {
    version: String,
    profiles: HashMap<String, Profile>,
}

fn get_profile(name: String, config_path: PathBuf) -> Profile {
    let config_path_str = config_path.to_string_lossy();

    println!("PELP attempting to get profile \"{}\" from {}", name, config_path_str);
    
    let conf = match fs::read_to_string(&config_path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("PELP failed to read {}: {e}", config_path_str);
            String::new()
        }
    };
    let conf: Config = match toml::de::from_str(&conf) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("PELP failed to parse config: {:#?}", e.message());
            Config::default()
        }
    };
    let profile = match conf.profiles.get(&name) {
        Some(x) => x,
        None => {
            eprintln!("PELP failed to find any profiles");
            &Vec::from([ColorEntry::default()])
        }
    };

    println!("PELP finished getting profile");

    profile.clone()
}