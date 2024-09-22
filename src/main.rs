mod config;

use std::{env, process};

use config::{load_config, Config};

fn get_binary_dirs(config: &Config) -> Vec<String> {
    // Load default paths
    let path_var = match env::var("PATH") {
        Ok(var) => var,
        Err(_) => {
            eprintln!("Failed to read PATH from environment");
            process::exit(1);
        }
    };

    let paths: Vec<String> = path_var.split(":").map(|entry| entry.to_string()).collect();
    [paths, config.extra_directories.clone()].concat()
}

fn main() -> Result<(), i32> {
    let config = load_config();
    let paths = get_binary_dirs(&config);

    for path in &paths {
        println!("{path}");
    }

    Ok(())
}
