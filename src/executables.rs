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
use regex::Regex;

use crate::config::Config;

#[derive(Debug, Clone, Eq)]
pub struct Executable {
    pub command: String,
    pub display_name: Option<String>,
}

impl PartialEq for Executable {
    fn eq(&self, other: &Self) -> bool {
        self.get_display_text().eq(other.get_display_text())
    }
}

impl Ord for Executable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_display_text().cmp(other.get_display_text())
    }
}

impl PartialOrd for Executable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_display_text()
            .partial_cmp(other.get_display_text())
    }
}

impl ToString for Executable {
    fn to_string(&self) -> String {
        match &self.display_name {
            Some(display_name) => format!("D:{} - {}", display_name, self.command),
            None => self.command.clone(),
        }
    }
}

impl Executable {
    fn new_binary(binary_name: String) -> Executable {
        Executable {
            command: binary_name,
            display_name: None,
        }
    }

    fn new_desktop_file(command: String, display_name: String) -> Executable {
        Executable {
            command,
            display_name: Some(display_name),
        }
    }

    pub fn get_display_text(&self) -> &str {
        match &self.display_name {
            Some(display_name) => display_name,
            None => &self.command,
        }
    }

    pub fn is_desktop_file(&self) -> bool {
        self.display_name.is_some()
    }
}

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

pub fn get_executables_for_config_and_paths(
    config: &Config,
    paths: &Vec<String>,
) -> Vec<Executable> {
    let mut executables: Vec<Executable>;
    if should_invalidate_cache(&config, &paths) {
        executables = paths
            .iter()
            .map(|path| {
                get_executables_from_directory(
                    path,
                    config.include_binaries,
                    config.include_desktop_files,
                )
            })
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
        match file.write_all(
            executables
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join("\n")
                .as_bytes(),
        ) {
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
            if entry.starts_with("D:") {
                let cached_info = entry.split(":").nth(1);
                if cached_info.is_none() {
                    error!("Cache entry for desktop file is corrupt: {entry}");
                    continue;
                }
                let executable_data: Vec<&str> = cached_info.unwrap().split(" - ").collect();
                if executable_data.len() != 2 {
                    error!("Cache entry for desktop file is corrupt: {entry}");
                    continue;
                }
                executables.push(Executable::new_desktop_file(
                    executable_data.get(1).unwrap().to_string(),
                    executable_data.get(0).unwrap().to_string(),
                ));
                continue;
            }
            if entry.contains(' ') {
                error!("Entry '{entry}' contains spaces.");
                continue;
            }
            executables.push(Executable::new_binary(entry));
        }
    }

    executables
}

fn get_executables_from_directory(
    dir: &str,
    include_binaries: bool,
    include_desktop_files: bool,
) -> Result<Vec<Executable>, ()> {
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

        let is_desktop_file = entry
            .path()
            .extension()
            .map(|e| e == "desktop")
            .unwrap_or(false);

        if !is_desktop_file && !include_binaries {
            continue;
        } else if is_desktop_file && include_desktop_files {
            let file = File::open(entry.path()).unwrap();
            let mut name: Option<String> = None;
            let mut exec: Option<String> = None;
            for line_result in BufReader::new(file).lines() {
                let entry = line_result.unwrap();
                if entry.starts_with("Name") {
                    name = Some(entry.split("=").nth(1).unwrap().to_string());
                } else if entry.starts_with("Exec") {
                    let entry = entry.split_once("=").unwrap().1;
                    let regex = Regex::new(r"%\w").unwrap();
                    exec = Some(regex.replace_all(entry, "").to_string());
                }
                if name.is_some() && exec.is_some() {
                    break;
                }
            }
            if name.is_none() || exec.is_none() {
                error!(
                    "Failed to get name and exec for desktop file '{}'",
                    entry.path().to_str().unwrap()
                );
                continue;
            }
            info!(
                "New desktop file: {} - {}",
                name.as_ref().unwrap(),
                exec.as_ref().unwrap()
            );
            executables.push(Executable::new_desktop_file(exec.unwrap(), name.unwrap()));
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
                let binary_name = Path::new(&entry.path().display().to_string())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                info!("New binary: {binary_name}");
                executables.push(Executable::new_binary(binary_name));
            }
        }
    }
    Ok(executables)
}
