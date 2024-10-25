mod app;
mod config;
mod executable_utils;

use app::app_main;

fn main() -> Result<(), i32> {
    env_logger::init();
    app_main();
    Ok(())
}
