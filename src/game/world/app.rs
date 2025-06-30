use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::DeviceEvent;
use log::{error, warn};

use crate::game::world::camera::Camera;
use crate::engine::graphics::{renderer::Renderer, texture::Texture};
use crate::game::world::chunk_manager::ChunkManager;
use crate::engine::input::InputHandler;

pub struct App {
    window: Option<Window>,
    size: Option<winit::dpi::PhysicalSize<u32>>,
    instance: Option<wgpu::Instance>,
    renderer: Option<Renderer>,
    camera: Camera,
    texture: Option<Texture>,
    chunk_manager: ChunkManager,
    atlas_helper: Option<crate::engine::graphics::texture::AtlasUVHelper>,
    input_handler: InputHandler,
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
            atlas_helper: None,
            input_handler: InputHandler::new(),
            fullscreen: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes())
            .map_err(|e| {
                error!("Failed to create window: {:?}", e);
                e
            }).unwrap_or_else(|_| {
                error!("Failed to create window, exiting");
                std::process::exit(1);
            });
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
                // Apply movement based on currently pressed keys
                self.input_handler.apply_movement(&mut self.camera);
                self.chunk_manager.update_chunks(self.camera.position);
                
                if let Some(atlas_helper) = &self.atlas_helper {
                    self.chunk_manager.poll_new_chunks(atlas_helper);
                }
                if let (Some(renderer), Some(texture)) = (&self.renderer, &self.texture) {
                    if let Some(window) = &self.window {
                        let instance = self.instance.as_ref().unwrap_or_else(|| {
                            error!("No wgpu instance available");
                            panic!("No wgpu instance available");
                        });
                        let surface = instance.create_surface(window).unwrap_or_else(|e| {
                            error!("Failed to create surface: {:?}", e);
                            panic!("Failed to create surface: {:?}", e);
                        });
                        surface.configure(&renderer.device, &renderer.config);
                        let chunks: Vec<&crate::game::world::chunk::Chunk> = self.chunk_manager.all_chunks().collect();
                        if let Err(e) = renderer.render(&surface, &self.camera, texture, &chunks) {
                            error!("Render error: {:?}", e);
                        }
                    }
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                    if event.state == winit::event::ElementState::Pressed {
                        self.input_handler.handle_keyboard_input_event(keycode, true);
                    } else if event.state == winit::event::ElementState::Released {
                        self.input_handler.handle_keyboard_input_event(keycode, false);
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                self.input_handler.handle_window_focus(focused, self.window.as_ref());
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input_handler.handle_mouse_motion(delta, &mut self.camera);
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

        let surface = instance.create_surface(window).unwrap_or_else(|e| {
            error!("Failed to create surface: {:?}", e);
            std::process::exit(1);
        });
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap_or_else(|| {
            error!("Failed to request adapter");
            std::process::exit(1);
        });

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap_or_else(|e| {
            error!("Failed to request device: {:?}", e);
            std::process::exit(1);
        });

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

        // Load texture atlas for blocks
        let texture_paths = [
            "assets/grass_block_top.png",   // 0
            "assets/grass_block_side.png", // 1
            "assets/dirt.png",             // 2
            "assets/stone.png",            // 3
        ];
        let texture = Texture::create_atlas_from_files(&device, &queue, &texture_paths)
            .unwrap_or_else(|e| {
                warn!("Failed to load texture atlas: {:?}, using default", e);
                Texture::create_default(&device, &queue)
            });

        // Create atlas helper for UV coordinate calculations
        let atlas_helper = crate::engine::graphics::texture::AtlasUVHelper::new(texture_paths.len());

        // Create renderer with owned device and queue
        let renderer = Renderer::new(device, queue, &surface, &adapter, size, &texture);

        self.instance = Some(instance);
        self.renderer = Some(renderer);
        self.texture = Some(texture);
        self.atlas_helper = Some(atlas_helper);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = Some(new_size);
            if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                let instance = self.instance.as_ref().unwrap_or_else(|| {
                    error!("No wgpu instance available for resize");
                    panic!("No wgpu instance available for resize");
                });
                let surface = instance.create_surface(window).unwrap_or_else(|e| {
                    error!("Failed to create surface for resize: {:?}", e);
                    panic!("Failed to create surface for resize: {:?}", e);
                });
                renderer.resize(new_size, &surface);
            }
        }
    }
} 