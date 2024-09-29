use display_info::DisplayInfo;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
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
        let window_pos_x = (display_width - window_width) / 2;
        let window_pos_y = (display_height - window_height) / 2;

        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(Size::Physical(PhysicalSize::new(
                            window_width,
                            window_height,
                        )))
                        .with_position(Position::Physical(PhysicalPosition::new(
                            window_pos_x as i32,
                            window_pos_y as i32,
                        )))
                        .with_resizable(false)
                        .with_decorations(false)
                        .with_title("MenuVroom")
                        .with_transparent(false),
                )
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                println!("Keyboard: {:?}", event.physical_key);
            }

            _ => {}
        }
    }
}

pub fn app_main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App { window: None };

    event_loop.run_app(&mut app).unwrap();
}
