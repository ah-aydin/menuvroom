mod app;
mod config;
mod executable;

use log::{error, info};
use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process};

use app::app_main;
use config::{load_config, Config};
use executable::Executable;

fn get_binary_dirs(config: &Config) -> Vec<String> {
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

fn get_executables(dir: &str) -> Result<Vec<Executable>, ()> {
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

        if file_type.is_file() || file_type.is_symlink() {
            let metadata = entry.metadata();
            if metadata.is_err() {
                error!("Failed to read metadata of entry '{:?}'", entry);
                continue;
            }

            let metadata = metadata.unwrap();
            let permissions = metadata.permissions();
            if permissions.mode() & 0o100 != 0 {
                executables.push(Executable::new(format!("{}", entry.path().display())));
            }
        }
    }
    Ok(executables)
}

fn main() -> Result<(), i32> {
    env_logger::init();

    let config = load_config();
    let paths = get_binary_dirs(&config);

    // TODO introduce caching
    let executables: Vec<Executable> = paths
        .iter()
        .map(|path| get_executables(path))
        .filter(|executables_result| executables_result.is_ok())
        .map(|executable_result| executable_result.unwrap())
        .flatten()
        .collect();

    app_main(executables);

    Ok(())
}
