static CONFIG_FILE_SUB_PATH: &'static str = "/.config/menuvroom/config.json";
static DEFAULT_CACHE_SUB_PATH: &'static str = "/.cache/menuvroom";
static DEFAULT_CONFIG: &'static str = r#"
{
  "extra_directories": [],
  "ignored_directories": [],
}
"#;

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::Path,
    process,
};

use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    extra_directories: Option<Vec<String>>,
    ignored_directories: Option<Vec<String>>,
    cache_dir: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub extra_directories: Vec<String>,
    pub ignored_directories: Vec<String>,
    pub cache_dir: String,
}

impl Config {
    pub fn new() -> Config {
        let home_var = env::var("HOME");
        if home_var.is_err() {
            error!("Failed to read HOME from environment");
            process::exit(1);
        }

        let config_file = home_var.unwrap() + CONFIG_FILE_SUB_PATH;

        // Create config file with default contents if it doesn't exist
        let config_file_path = Path::new(&config_file);
        if !config_file_path.exists() {
            info!("Config file missing, creating new config file with default contents");

            match config_file_path.parent() {
                Some(parent) => match fs::create_dir_all(parent) {
                    Err(_) => {
                        error!("Failed to create missing parent directory for '{config_file}'");
                        process::exit(1);
                    }
                    _ => {}
                },
                None => {
                    error!("Could not get the parent directory for '{config_file}'");
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
                    error!("Failed to create missing config file '{config_file}'");
                    process::exit(1);
                }
            };

            match file.write_all(DEFAULT_CONFIG.as_bytes()) {
                Err(_) => {
                    error!("Failed to write default contents to newly created config file");
                    process::exit(1);
                }
                _ => {}
            };
        }

        // Read and parse config
        let file = match std::fs::File::open(&config_file) {
            Ok(file) => file,
            Err(_) => {
                error!("Could not open config file '{}'", config_file);
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
                error!("Failed to read config file");
                process::exit(1);
            }
        };

        let config_file: Result<ConfigFile, serde_json::Error> = serde_json::from_str(&config_raw);
        if config_file.is_err() {
            error!("Failed to parse config file");
            std::process::exit(1);
        }
        let config_file = config_file.unwrap();

        let extra_directories = config_file.extra_directories.unwrap_or(vec![]);
        let ignored_directories = config_file.ignored_directories.unwrap_or(vec![]);
        let cache_dir = config_file
            .cache_dir
            .or_else(|| {
                let home_var = env::var("HOME");
                if home_var.is_err() {
                    error!("Failed to read HOME from environment");
                    process::exit(1);
                }

                let cache_dir = home_var.unwrap() + DEFAULT_CACHE_SUB_PATH;
                Some(cache_dir)
            })
            .unwrap();
        Config {
            extra_directories,
            ignored_directories,
            cache_dir,
        }
    }
}
