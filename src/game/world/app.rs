use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId, Fullscreen};
use winit::keyboard::KeyCode;
use winit::event::DeviceEvent;

use crate::game::world::camera::Camera;
use crate::engine::graphics::{renderer::Renderer, texture::Texture};
use crate::game::world::chunk_manager::ChunkManager;

pub struct App {
    window: Option<Window>,
    size: Option<winit::dpi::PhysicalSize<u32>>,
    instance: Option<wgpu::Instance>,
    renderer: Option<Renderer>,
    camera: Camera,
    texture: Option<Texture>,
    chunk_manager: ChunkManager,
    fullscreen: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            size: None,
            instance: None,
            renderer: None,
            camera: Camera::new(),
            texture: None,
            chunk_manager: ChunkManager::new(10), // view_distance = 1 for now
            fullscreen: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        let size = window.inner_size();
        self.size = Some(size);
        self.window = Some(window);
        // Initialize wgpu
        pollster::block_on(self.init_wgpu());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.chunk_manager.poll_new_chunks();
                if let (Some(renderer), Some(texture)) = (&self.renderer, &self.texture) {
                    if let Some(window) = &self.window {
                        let instance = self.instance.as_ref().unwrap();
                        let surface = instance.create_surface(window).unwrap();
                        surface.configure(&renderer.device, &renderer.config);
                        let chunks: Vec<&crate::game::world::chunk::Chunk> = self.chunk_manager.all_chunks().collect();
                        if let Err(e) = renderer.render(&surface, &self.camera, texture, &chunks) {
                            eprintln!("Render error: {:?}", e);
                        }
                    }
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                        match keycode {
                            KeyCode::KeyW => self.camera.move_forward(),
                            KeyCode::KeyS => self.camera.move_backward(),
                            KeyCode::KeyA => self.camera.move_left(),
                            KeyCode::KeyD => self.camera.move_right(),
                            KeyCode::Space => self.camera.fly_up(),
                            KeyCode::ShiftLeft => self.camera.fly_down(),
                            KeyCode::ShiftRight => self.camera.fly_down(),
                            KeyCode::F11 => {
                                if let Some(window) = &self.window {
                                    if self.fullscreen {
                                        window.set_fullscreen(None);
                                    } else {
                                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                    }
                                    self.fullscreen = !self.fullscreen;
                                }
                            },
                            _ => {}
                        }
                        self.chunk_manager.update_chunks(self.camera.position);
                    }
                }
            }
            WindowEvent::Focused(true) => {
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                    window.set_cursor_visible(false);
                }
            }
            WindowEvent::Focused(false) => {
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let sensitivity = 0.002;
            self.camera.rotate(delta.0 as f32 * sensitivity, -delta.1 as f32 * sensitivity);
        }
    }
}

impl App {
    async fn init_wgpu(&mut self) {
        let window = self.window.as_ref().unwrap();
        let size = self.size.unwrap();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap();

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Load texture array for blocks
        let texture_paths = [
            "assets/grass_block_top.png",   // 0
            "assets/grass_block_side.png", // 1
            "assets/dirt.png",             // 2
            "assets/stone.png",            // 3
        ];
        let texture = Texture::load_array(&device, &queue, &texture_paths)
            .unwrap_or_else(|_| Texture::create_default(&device, &queue));

        // Create renderer with owned device and queue
        let renderer = Renderer::new(device, queue, &surface, &adapter, size, &texture);

        self.instance = Some(instance);
        self.renderer = Some(renderer);
        self.texture = Some(texture);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = Some(new_size);
            if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                let instance = self.instance.as_ref().unwrap();
                let surface = instance.create_surface(window).unwrap();
                renderer.resize(new_size, &surface);
            }
        }
    }
} 