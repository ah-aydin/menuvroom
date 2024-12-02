mod app;
mod config;
mod executables;

use app::app_main;

fn main() -> Result<(), i32> {
    env_logger::init();
    app_main();
    Ok(())
}
