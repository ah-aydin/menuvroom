use std::{io, os::unix::process::CommandExt, path::Path, process::Command, sync::Arc};

use glyphon::TextArea;
use log::{error, info};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::NamedKey,
    window::Window,
};

use crate::config::Config;
use crate::executable_utils;

struct AppState {
    search_entry: String,
    config: Config,
    paths: Vec<String>,
    executables: Vec<String>,
    matching_executable_indexes: Vec<usize>,
    selected_index: usize,
    ctrl_pressed: bool,
}

impl AppState {
    fn new(config: Config, paths: Vec<String>, executables: Vec<String>) -> Self {
        Self {
            search_entry: String::with_capacity(255),
            config,
            paths,
            executables,
            matching_executable_indexes: Vec::with_capacity(8),
            selected_index: 0,
            ctrl_pressed: false,
        }
    }

    fn append_to_search(&mut self, s: &str) {
        self.search_entry.push_str(s);
        self.update_matching_executable_indexes();
    }

    fn search_backspace(&mut self) {
        self.search_entry.pop();
        self.update_matching_executable_indexes();
    }

    fn update_matching_executable_indexes(&mut self) {
        self.selected_index = 0;
        self.matching_executable_indexes.clear();

        if self.search_entry.is_empty() {
            return;
        }

        for i in 0..self.executables.len() {
            let display_name = &self.executables[i];
            if *display_name == self.search_entry {
                self.matching_executable_indexes.insert(0, i);
            }
            if display_name.contains(&self.search_entry) {
                self.matching_executable_indexes.push(i);
            }
        }

        info!("The executables are:");
        let mut c = 0;
        for i in &self.matching_executable_indexes {
            info!("{}: {}", c, self.executables[*i]);
            c += 1;
        }
    }

    fn increment_selected_index(&mut self) {
        self.selected_index =
            (self.selected_index + 1).min(self.matching_executable_indexes.len() - 1);
        info!("Selected index: {}", self.selected_index);
    }

    fn decrement_selected_index(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        info!("selected index: {}", self.selected_index);
    }

    fn get_selected_executable(&self) -> Option<&str> {
        if self.selected_index >= self.matching_executable_indexes.len() {
            return None;
        }
        Some(&self.executables[self.matching_executable_indexes[self.selected_index]])
    }

    fn get_executable(&self, index: usize) -> Option<&str> {
        if index < self.matching_executable_indexes.len() {
            return Some(&self.executables[self.matching_executable_indexes[index]]);
        }
        None
    }

    fn get_text_buffers(
        &self,
        font_system: &mut glyphon::FontSystem,
        width: f32,
        height: f32,
    ) -> Vec<glyphon::Buffer> {
        let font = glyphon::Family::Monospace;
        let font_size = self.config.font_size;
        let line_height = self.config.line_height;

        let mut text_buffers = Vec::with_capacity(self.matching_executable_indexes.len() + 1);

        let mut search_entry_text_buffer =
            glyphon::Buffer::new(font_system, glyphon::Metrics::new(font_size, line_height));

        search_entry_text_buffer.set_size(font_system, Some(width), Some(height));
        search_entry_text_buffer.set_text(
            font_system,
            &self.search_entry,
            glyphon::Attrs::new().family(font),
            glyphon::Shaping::Advanced,
        );
        search_entry_text_buffer.shape_until_scroll(font_system, false);
        text_buffers.push(search_entry_text_buffer);

        fn get_index_hint(i: usize) -> String {
            if i <= 8 {
                return format!("(Ctrl+{})", i + 1);
            } else if i == 9 {
                return format!("(Ctrl+0)");
            } else {
                return "".to_string();
            }
        }

        for i in 0..self.matching_executable_indexes.len() {
            let index = self.matching_executable_indexes[i];
            let executable = &self.executables[index];

            let mut text_buffer =
                glyphon::Buffer::new(font_system, glyphon::Metrics::new(font_size, line_height));

            text_buffer.set_size(font_system, Some(width), Some(height));
            text_buffer.set_text(
                font_system,
                &format!("{} {}", executable, get_index_hint(i)),
                glyphon::Attrs::new().family(font),
                glyphon::Shaping::Advanced,
            );
            text_buffer.shape_until_scroll(font_system, false);
            text_buffers.push(text_buffer);
        }

        text_buffers
    }
}

struct WindowState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,

    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,

    window: Arc<Window>,
}

impl WindowState {
    async fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();

        // Set up surface
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to create adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to request device");

        let surface_capabilities = surface.get_capabilities(&adapter);

        let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Set up text renderer
        let font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas = glyphon::TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            window,
        }
    }
}

struct App {
    state: AppState,
    window_state: Option<WindowState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Config {
            window_width,
            window_height,
            window_pos_x,
            window_pos_y,
            ..
        } = self.state.config;
        info!("Creating window with size ({window_width}, {window_height}) at position ({window_pos_x}, {window_pos_y})");
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(window_width, window_height))
            .with_position(LogicalPosition::new(window_pos_x, window_pos_y))
            .with_title("Menu Vroom")
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true);
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );
        self.window_state = Some(pollster::block_on(WindowState::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(window_state) = &mut self.window_state else {
            return;
        };

        let WindowState {
            window,
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            ..
        } = window_state;

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                viewport.update(
                    &queue,
                    glyphon::Resolution {
                        width: surface_config.width,
                        height: surface_config.height,
                    },
                );

                let physical_width =
                    (window.inner_size().width as f64 * window.scale_factor()) as f32;
                let physical_height =
                    (window.inner_size().height as f64 * window.scale_factor()) as f32;

                let mut text_areas = Vec::new();
                let text_buffers =
                    self.state
                        .get_text_buffers(font_system, physical_width, physical_height);
                let mut top = 10.0;
                let mut index = 0;
                for text_buffer in &text_buffers {
                    let color;
                    if index - 1 == self.state.selected_index {
                        color = self.state.config.font_color_highlighted;
                    } else {
                        color = self.state.config.font_color;
                    }
                    text_areas.push(TextArea {
                        buffer: text_buffer,
                        left: 10.0,
                        top,
                        scale: 1.0,
                        bounds: glyphon::TextBounds {
                            left: 0,
                            top: 0,
                            right: physical_width as i32,
                            bottom: physical_height as i32,
                        },
                        default_color: color,
                        custom_glyphs: &[],
                    });
                    top += self.state.config.line_height;
                    index += 1;
                }

                text_renderer
                    .prepare(
                        device,
                        queue,
                        font_system,
                        atlas,
                        viewport,
                        text_areas,
                        swash_cache,
                    )
                    .expect("Failed to prepare text renderer");

                let frame = surface
                    .get_current_texture()
                    .expect("Faield to get current texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.state.config.bg_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                text_renderer
                    .render(&atlas, &viewport, &mut pass)
                    .expect("Failed to render text");
                drop(pass);

                queue.submit(Some(encoder.finish()));
                frame.present();
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.state.is_pressed() {
                    match event.logical_key {
                        winit::keyboard::Key::Named(NamedKey::Control) => {
                            self.state.ctrl_pressed = true;
                        }

                        winit::keyboard::Key::Named(NamedKey::Enter) => {
                            if let Some(executable) = self.state.get_selected_executable() {
                                run_executable(&self.state.paths, executable);
                            }
                            event_loop.exit();
                        }
                        winit::keyboard::Key::Named(NamedKey::Escape) => {
                            event_loop.exit();
                        }

                        winit::keyboard::Key::Named(NamedKey::ArrowUp) => {
                            self.state.decrement_selected_index();
                        }
                        winit::keyboard::Key::Named(NamedKey::ArrowDown) => {
                            self.state.increment_selected_index();
                        }

                        winit::keyboard::Key::Named(NamedKey::Backspace) => {
                            self.state.search_backspace()
                        }
                        winit::keyboard::Key::Named(NamedKey::Space) => {
                            self.state.append_to_search(" ")
                        }

                        winit::keyboard::Key::Character(c) => {
                            if self.state.ctrl_pressed {
                                let executable = match c.as_str() {
                                    "1" => self.state.get_executable(0),
                                    "2" => self.state.get_executable(1),
                                    "3" => self.state.get_executable(2),
                                    "4" => self.state.get_executable(3),
                                    "5" => self.state.get_executable(4),
                                    "6" => self.state.get_executable(5),
                                    "7" => self.state.get_executable(6),
                                    "8" => self.state.get_executable(7),
                                    "9" => self.state.get_executable(8),
                                    "0" => self.state.get_executable(9),
                                    _ => None,
                                };
                                if let Some(executable) = executable {
                                    run_executable(&self.state.paths, executable);
                                    event_loop.exit();
                                }
                            } else {
                                self.state.append_to_search(c.as_str())
                            }
                        }
                        _ => {}
                    };

                    window.request_redraw();
                } else {
                    match event.logical_key {
                        winit::keyboard::Key::Named(NamedKey::Control) => {
                            self.state.ctrl_pressed = false;
                        }
                        _ => {}
                    };
                }
            }

            _ => {}
        }
    }
}

fn run_executable(directories: &Vec<String>, executable: &str) {
    let mut command = None;
    for dir in directories {
        let full_path = format!("{}/{}", dir.clone(), executable);
        if Path::new(&full_path).exists() {
            command = Some(full_path);
            break;
        }
    }
    if command.is_none() {
        error!("Failed to find executable");
        std::process::exit(1);
    }

    let command = command.unwrap();
    info!("Launching: {}", command);
    let r = unsafe {
        Command::new(command)
            .pre_exec(|| {
                nix::unistd::setsid().map_err(|_| io::Error::from(io::ErrorKind::Other))?;
                Ok(())
            })
            .spawn()
    };
    if r.is_err() {
        info!("Failed to spawn process");
    }
}

pub fn app_main() {
    let config = Config::new();
    let paths = executable_utils::get_binary_dirs(&config);

    let executables = executable_utils::get_executables_for_config_and_paths(&config, &paths);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        state: AppState::new(config, paths, executables),
        window_state: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
