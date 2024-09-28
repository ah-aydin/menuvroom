#![allow(non_snake_case)]

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_desktop::tao::dpi::LogicalPosition;
use dioxus_desktop::tao::dpi::LogicalSize;
use display_info::DisplayInfo;

fn Stories() -> Element {
    rsx! {
        h2 { "stories" }
    }
}

fn Preview() -> Element {
    rsx! {
        h2 { "preview" }
    }
}

fn App() -> Element {
    eval(
        r#"
        document.addEventListener('contextmenu', event => event.preventDefault());
        "#,
    );
    rsx! {
        div { display: "flex", flex_direction: "row", width: "100%",
            div { width: "50%", Stories {} }
            div { width: "50%", Preview {} }
        }
    }
}

pub fn app_main() {
    let display_infos = match DisplayInfo::all() {
        Ok(display_infos) => display_infos,
        Err(err) => {
            println!("Failed to get display informations");
            println!("{:?}", err);
            std::process::exit(1);
        }
    };
    let primary_displays = display_infos
        .iter()
        .filter(|display_info| display_info.is_primary)
        .collect::<Vec<&DisplayInfo>>();
    let primary_display_info = match primary_displays.first() {
        Some(display_info) => display_info,
        None => {
            println!("Failed to locate primary display. {:?}", display_infos);
            std::process::exit(1);
        }
    };

    let display_width = primary_display_info.width;
    let display_height = primary_display_info.height;

    let window_width = (display_width as f32 * 0.75) as u32;
    let window_height = (display_height as f32 * 0.75) as u32;
    let window_pos_x = (display_width - window_width )/ 2;
    let window_pos_y = (display_height - window_height) / 2;

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("menuvroom")
                        .with_inner_size(LogicalSize::new(window_width, window_height))
                        .with_position(LogicalPosition::new(window_pos_x, window_pos_y))
                        .with_focused(true)
                        .with_decorations(false)
                        .with_transparent(false)
                        .with_always_on_top(true)
                        .with_fullscreen(None)
                        .with_resizable(false)
                        .with_closable(true)
                        .with_minimizable(false)
                        .with_maximized(false)
                        .with_theme(None),
                )
                .with_menu(None),
        )
        .launch(App);
}
