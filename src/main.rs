mod config;

use std::{env, fs, io, os::unix::fs::PermissionsExt, path::Path, process};

use config::{load_config, Config};

fn get_binary_dirs(config: &Config) -> Vec<String> {
    let path_var = match env::var("PATH") {
        Ok(var) => var,
        Err(_) => {
            eprintln!("Failed to read PATH from environment");
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

// TODO improve error handling
fn list_executables(dir: &str) -> io::Result<()> {
    let path = Path::new(dir);
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_file() {
            let metadata = entry.metadata()?;
            let permissions = metadata.permissions();

            if permissions.mode() & 0o100 != 0 {
                println!("{}", entry.path().display());
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), i32> {
    let config = load_config();
    let paths = get_binary_dirs(&config);

    for path in &paths {
        if list_executables(&path).is_err() {
            eprintln!("Failed to read executables files in {path}");
        }
    }

    Ok(())
}
