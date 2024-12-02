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
struct FontColor {
    r: u8,
    g: u8,
    b: u8,
}

impl FontColor {
    fn to_glyphon_color(&self) -> glyphon::Color {
        glyphon::Color::rgb(self.r, self.g, self.b)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BgColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl BgColor {
    fn to_wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    extra_directories: Option<Vec<String>>,
    ignored_directories: Option<Vec<String>>,
    cache_dir: Option<String>,

    include_binaries: Option<bool>,

    window_width: Option<u32>,
    window_height: Option<u32>,
    window_pos_x: Option<i32>,
    window_pos_y: Option<i32>,

    font_color: Option<FontColor>,
    font_color_highlighted: Option<FontColor>,
    font_size: Option<f32>,
    line_height: Option<f32>,

    bg_color: Option<BgColor>,
}

#[derive(Debug)]
pub struct Config {
    pub extra_directories: Vec<String>,
    pub ignored_directories: Vec<String>,
    pub cache_dir: String,

    pub include_binaries: bool,

    pub window_width: u32,
    pub window_height: u32,
    pub window_pos_x: i32,
    pub window_pos_y: i32,

    pub font_color: glyphon::Color,
    pub font_color_highlighted: glyphon::Color,
    pub font_size: f32,
    pub line_height: f32,

    pub bg_color: wgpu::Color,
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
            extra_directories: config_file.extra_directories.unwrap_or(vec![]),
            ignored_directories: config_file.ignored_directories.unwrap_or(vec![]),
            cache_dir,

            include_binaries: config_file.include_binaries.unwrap_or(true),

            // These values are for a 1080p display to cover 2 thirds of the screen
            window_width: config_file.window_width.unwrap_or(1440),
            window_height: config_file.window_height.unwrap_or(810),
            window_pos_x: config_file.window_pos_x.unwrap_or(240),
            window_pos_y: config_file.window_pos_y.unwrap_or(135),

            font_color: config_file
                .font_color
                .map(|fc| fc.to_glyphon_color())
                .unwrap_or(glyphon::Color::rgb(255, 255, 255)),
            font_color_highlighted: config_file
                .font_color_highlighted
                .map(|fc| fc.to_glyphon_color())
                .unwrap_or(glyphon::Color::rgb(255, 0, 0)),
            font_size: config_file.font_size.unwrap_or(30.0),
            line_height: config_file.font_size.unwrap_or(42.0),

            bg_color: config_file
                .bg_color
                .map(|bgc| bgc.to_wgpu_color())
                .unwrap_or(wgpu::Color {
                    r: 0.15,
                    g: 0.15,
                    b: 0.15,
                    a: 0.8,
                }),
        }
    }
}
