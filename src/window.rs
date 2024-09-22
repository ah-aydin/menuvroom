use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{self, HasDisplayHandle, HasRawDisplayHandle};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

pub fn window_main() {
    let event_loop = EventLoop::new().unwrap();
    let binding = event_loop.display_handle().unwrap();
    let l = binding.as_raw();
    match l {
        raw_window_handle::RawDisplayHandle::Xlib(xlib_display_handle) => {
            println!("{:?}", xlib_display_handle);
            let l = xlib_display_handle.display.unwrap();
            println!("Display: {:?}", l);
        }
        raw_window_handle::RawDisplayHandle::Wayland(wayland_display_handle) => {
            println!("{:?}", wayland_display_handle);
        }
        _ => unreachable!("Unsupported display manager"),
    }

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
