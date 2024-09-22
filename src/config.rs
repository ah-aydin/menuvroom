static CONFIG_FILE_SUB_PATH: &'static str = "/.config/rmenu/config.json";
static DEFAULT_CONFIG: &'static str = r#"
{
  "extra_directories": []
}
"#;

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::Path,
    process,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub extra_directories: Vec<String>,
}

pub fn load_config() -> Config {
    let home_var = env::var("HOME");
    if home_var.is_err() {
        eprintln!("Failed to read HOME from environment");
        process::exit(1);
    }

    let config_file = home_var.unwrap() + CONFIG_FILE_SUB_PATH;

    // Create config file with default contents if it doesn't exist
    let config_file_path = Path::new(&config_file);
    if !config_file_path.exists() {
        println!("Config file missing, creating new config file with default contents");

        match config_file_path.parent() {
            Some(parent) => match fs::create_dir_all(parent) {
                Err(_) => {
                    eprintln!("Failed to create missing parent directory for '{config_file}'");
                    process::exit(1);
                }
                _ => {}
            },
            None => {
                eprintln!("Could not get the parent directory for '{config_file}'");
                process::exit(1);
            }
        };

        let mut file = match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&config_file)
        {
            Ok(file) => file,
            Err(_) => {
                println!("Failed to create missing config file '{config_file}'");
                process::exit(1);
            }
        };

        match file.write_all(DEFAULT_CONFIG.as_bytes()) {
            Err(_) => {
                eprintln!("Failed to write default contents to newly created config file");
                process::exit(1);
            }
            _ => {}
        };
    }

    // Read and parse config
    let file = match std::fs::File::open(&config_file) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Could not open config file '{}'", config_file);
            process::exit(1);
        }
    };

    let config_raw = match BufReader::new(file)
        .lines()
        .map(|l| l.unwrap())
        .into_iter()
        .reduce(|acc, l| acc + &l)
    {
        Some(acc) => acc,
        None => {
            eprintln!("Failed to read config file");
            process::exit(1);
        }
    };

    let config: Result<Config, serde_json::Error> = serde_json::from_str(&config_raw);
    match config {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to parse config file: {:?}", err);
            process::exit(1);
        }
    }
}
