pub static CACHE_FILE_NAME: &'static str = "/executables.txt";

use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    process,
};

use log::{error, info};

use crate::config::Config;

pub fn get_binary_dirs(config: &Config) -> Vec<String> {
    let path_var = match env::var("PATH") {
        Ok(var) => var,
        Err(_) => {
            error!("Failed to read PATH from environment");
            process::exit(1);
        }
    };

    let paths: Vec<String> = path_var
        .split(":")
        .map(|entry| entry.to_string())
        .filter(|path| fs::exists(path).unwrap_or(false))
        .filter(|path| !config.ignored_directories.contains(&path))
        .collect();
    let paths = [paths, config.extra_directories.clone()].concat();

    let mut paths = paths
        .iter()
        .map(|path| path.to_string())
        .collect::<Vec<String>>();
    paths.sort();
    paths.dedup();
    paths
}

fn should_invalidate_cache(config: &Config, executable_dirs: &Vec<String>) -> bool {
    let cache_file = config.cache_dir.clone() + CACHE_FILE_NAME;
    let cache_file_path = Path::new(&cache_file);

    // If cache files does not exist create it
    if !cache_file_path.exists() {
        info!("Cache file does not exist, creating a new one");
        match cache_file_path.parent() {
            Some(parent) => match fs::create_dir_all(parent) {
                Err(_) => {
                    error!("Failed to create missing parent directory for '{cache_file}'");
                    process::exit(1);
                }
                _ => {}
            },
            None => {
                error!("Could not get parent directory for '{cache_file}'");
                process::exit(1);
            }
        }
        match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&cache_file)
        {
            Err(err) => {
                error!("{:?}", err);
                error!("Failed to create missing cache file '{cache_file}'");
                process::exit(1);
            }
            _ => {}
        };
        return true;
    }

    // If cache file is not up-to-date
    let cache_last_update_time = cache_file_path.metadata().unwrap().mtime();
    for executable_dir in executable_dirs {
        let path = Path::new(executable_dir);
        if !path.exists() {
            continue;
        }
        if path.metadata().unwrap().mtime() > cache_last_update_time {
            info!("Cache file is not up-to-date");
            return true;
        }
    }

    info!("Cache file is up-to-date");
    false
}

pub fn get_executables_for_config_and_paths(config: &Config, paths: &Vec<String>) -> Vec<String> {
    let mut executables: Vec<String>;
    if should_invalidate_cache(&config, &paths) {
        executables = paths
            .iter()
            .map(|path| get_executables_from_directory(path, config.include_binaries))
            .filter(|executables_result| executables_result.is_ok())
            .map(|executable_result| executable_result.unwrap())
            .flatten()
            .collect();
        executables.sort();
        executables.dedup();
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(config.cache_dir.clone() + CACHE_FILE_NAME)
            .unwrap();
        match file.write_all(executables.join("\n").as_bytes()) {
            Err(_) => {
                error!("Failed to update cache file");
                process::exit(1);
            }
            _ => {}
        };
    } else {
        executables = vec![];
        let file = File::open(config.cache_dir.clone() + CACHE_FILE_NAME).unwrap();
        for line_result in BufReader::new(file).lines() {
            let entry = line_result.unwrap();
            if entry.contains(' ') {
                error!("Entry '{entry}' contains spaces.");
                continue;
            }
            executables.push(entry);
        }
    }

    executables
}

fn get_executables_from_directory(dir: &str, include_binaries: bool) -> Result<Vec<String>, ()> {
    info!("Collecting from dir: {}", dir);

    let path = Path::new(dir);
    let entries = fs::read_dir(path);
    if entries.is_err() {
        error!("Failed to read entries in '{}'", dir);
        return Err(());
    }
    let entries = entries.unwrap();

    let mut executables = Vec::with_capacity(entries.size_hint().0);

    for entry in entries {
        if entry.is_err() {
            error!(
                "Failed to read details about entry '{:?}'",
                entry.err().unwrap()
            );
            continue;
        }
        let entry = entry.unwrap();

        let file_type = entry.file_type();
        if file_type.is_err() {
            error!("Could not get filetype of '{:?}'", entry);
            continue;
        }
        let file_type = file_type.unwrap();

        if !entry.path().ends_with(".desktop") && !include_binaries {
            continue;
        }

        if file_type.is_file() || file_type.is_symlink() {
            let metadata = entry.metadata();
            if metadata.is_err() {
                error!("Failed to read metadata of entry '{:?}'", entry);
                continue;
            }

            let metadata = metadata.unwrap();
            let permissions = metadata.permissions();
            if permissions.mode() & 0o100 != 0 {
                executables.push(
                    Path::new(&entry.path().display().to_string())
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
        }
    }
    Ok(executables)
}
