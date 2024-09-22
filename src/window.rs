#![allow(non_snake_case)]

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_desktop::tao::dpi::LogicalPosition;
use dioxus_desktop::tao::dpi::LogicalSize;

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

pub fn window_main() {
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("menuvroom")
                        // TODO compute size according to display resolution
                        .with_inner_size(LogicalSize::new(800, 600))
                        // TODO compute position according to the window size and display resolution
                        .with_position(LogicalPosition::new(300, 200))
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
