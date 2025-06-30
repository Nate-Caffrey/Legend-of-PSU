use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::DeviceEvent;
use log::{error, warn};
use std::time::Instant;

use crate::engine::window::WindowManager;
use crate::engine::graphics::{renderer::Renderer, texture::Texture};
use crate::game::world::chunk_manager::ChunkManager;
use crate::game::state::GameState;
use crate::game::player::Player;
use crate::engine::input::InputHandler;

pub struct App {
    window_manager: WindowManager,
    instance: Option<wgpu::Instance>,
    renderer: Option<Renderer>,
    player: Player,
    texture: Option<Texture>,
    chunk_manager: ChunkManager,
    atlas_helper: Option<crate::engine::graphics::texture::AtlasUVHelper>,
    game_state: GameState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window_manager: WindowManager::new(),
            instance: None,
            renderer: None,
            player: Player::new(),
            texture: None,
            chunk_manager: ChunkManager::new(10), // view_distance = 10 for now
            atlas_helper: None,
            game_state: GameState::new(),
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
        self.window_manager.set_window(window);
        // Initialize wgpu
        pollster::block_on(self.init_wgpu());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Update player movement
                self.player.update(0.016); // Assuming 60 FPS for now
                self.chunk_manager.update_chunks(self.player.get_position());
                
                if let Some(renderer) = &self.renderer {
                    self.chunk_manager.poll_new_chunks(&renderer.device);
                }
                if let (Some(renderer), Some(texture)) = (&self.renderer, &self.texture) {
                    if let Some(window) = self.window_manager.get_window() {
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
                        if let Err(e) = renderer.render(&surface, self.player.get_camera(), texture, &chunks, &self.chunk_manager) {
                            error!("Render error: {:?}", e);
                        }
                    }
                }
                
                // Update game state (FPS tracking)
                self.game_state.update_frame_count();
                if let Some(fps) = self.game_state.update_fps_display() {
                    println!("FPS: {}", fps);
                }
                
                self.window_manager.request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                    let pressed = event.state == winit::event::ElementState::Pressed;
                    if pressed && keycode == winit::keyboard::KeyCode::F3 {
                        self.game_state.toggle_fps_display();
                    }
                    self.player.handle_keyboard_input(keycode, pressed);
                }
            }
            WindowEvent::Focused(focused) => {
                self.player.handle_window_focus(focused, self.window_manager.get_window());
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        self.player.handle_device_event(event);
    }
}

impl App {
    async fn init_wgpu(&mut self) {
        let window = self.window_manager.window.as_ref().unwrap();
        let size = window.inner_size();

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
            self.window_manager.set_window_size(new_size);
            if let (Some(renderer), Some(window)) = (&mut self.renderer, self.window_manager.get_window()) {
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