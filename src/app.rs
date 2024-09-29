use std::sync::Arc;

use display_info::DisplayInfo;
use log::info;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::NamedKey,
    window::Window,
};

use crate::executable::Executable;

struct WindowOptions {
    window_width: u32,
    window_height: u32,
    window_pos_x: i32,
    window_pos_y: i32,
}

impl WindowOptions {
    fn new() -> Self {
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
        let window_pos_x = ((display_width - window_width) / 2) as i32;
        let window_pos_y = ((display_height - window_height) / 2) as i32;

        Self {
            window_width,
            window_height,
            window_pos_x,
            window_pos_y,
        }
    }
}

struct Gpu<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
}

impl<'window> Gpu<'window> {
    async fn new(window: Arc<Window>, width: u32, height: u32) -> Gpu<'window> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Faeild to request adapter!");

        let (device, queue) = {
            info!("WGPU Adapter features: {:#?}", adapter.features());
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("WGPU Device"),
                        required_features: wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::default(),
                    },
                    None,
                )
                .await
                .expect("Failed to request a device!")
        };

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Self {
            surface,
            device,
            queue,
            surface_config,
            surface_format,
        }
    }

    fn create_depth_texture(&self, width: u32, height: u32) -> wgpu::TextureView {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            array_layer_count: None,
            mip_level_count: None,
        })
    }
}

struct Renderer<'window> {
    gpu: Gpu<'window>,
    depth_texture_view: wgpu::TextureView,
}

impl<'window> Renderer<'window> {
    async fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let gpu = Gpu::new(window, width, height).await;
        let depth_texture_view = gpu.create_depth_texture(width, height);

        Self {
            gpu,
            depth_texture_view,
        }
    }

    fn clear(&self) {
        let surface_texture = self
            .gpu
            .surface
            .get_current_texture()
            .expect("Failed to get surface texture");

        let surface_texture_view =
            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: wgpu::Label::default(),
                    aspect: wgpu::TextureAspect::default(),
                    format: Some(self.gpu.surface_format),
                    dimension: None,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                });

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Clear encoder"),
            });
        encoder.insert_debug_marker("Clear window");

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.15,
                        a: 0.8,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        drop(render_pass);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

struct AppState {
    search_entry: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            search_entry: String::with_capacity(255),
        }
    }

    fn append_to_search(&mut self, s: &str) {
        self.search_entry.push_str(s);
    }

    fn search_backspace(&mut self) {
        self.search_entry.pop();
    }
}

struct App<'window> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'window>>,

    state: AppState,

    window_options: WindowOptions,
    _executables: Vec<Executable>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_inner_size(Size::Physical(PhysicalSize::new(
                                self.window_options.window_width,
                                self.window_options.window_height,
                            )))
                            .with_position(Position::Physical(PhysicalPosition::new(
                                self.window_options.window_pos_x,
                                self.window_options.window_pos_y,
                            )))
                            .with_resizable(false)
                            .with_decorations(false)
                            .with_title("MenuVroom")
                            .with_transparent(true),
                    )
                    .unwrap(),
            );
            self.window = Some(window.clone());

            let window_width = self.window_options.window_width;
            let window_height = self.window_options.window_height;
            let renderer = pollster::block_on(async move {
                Renderer::new(window.clone(), window_width, window_height).await
            });

            self.renderer = Some(renderer);
        }
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
                self.renderer.as_ref().unwrap().clear();
                //self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.state.is_pressed() {
                    match event.logical_key {
                        winit::keyboard::Key::Named(NamedKey::Enter) => {
                            info!("Pressed the enter key. TODO open up the selected app");
                        }
                        winit::keyboard::Key::Named(NamedKey::Backspace) => {
                            self.state.search_backspace()
                        }
                        winit::keyboard::Key::Named(NamedKey::Space) => {
                            self.state.append_to_search(" ")
                        }
                        winit::keyboard::Key::Character(c) => {
                            self.state.append_to_search(c.as_str())
                        }
                        _ => {}
                    };
                    info!("Search: {}", self.state.search_entry);
                }
            }

            _ => {}
        }
    }
}

pub fn app_main(executables: Vec<Executable>) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        window: None,
        renderer: None,

        state: AppState::new(),

        window_options: WindowOptions::new(),
        _executables: executables,
    };

    event_loop.run_app(&mut app).unwrap();
}
