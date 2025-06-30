//! Player implementation.

use crate::game::world::camera::Camera;
use crate::engine::input::InputHandler;
use winit::event::DeviceEvent;
use winit::window::Window;

pub struct Player {
    pub camera: Camera,
    pub input_handler: InputHandler,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            input_handler: InputHandler::new(),
            movement_speed: 5.0,
            mouse_sensitivity: 0.002,
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // Apply movement based on currently pressed keys
        self.input_handler.apply_movement(&mut self.camera);
    }

    pub fn handle_mouse_motion(&mut self, delta: winit::dpi::PhysicalPosition<f64>) {
        let delta_tuple = (delta.x, delta.y);
        self.input_handler.handle_mouse_motion(delta_tuple, &mut self.camera);
    }

    pub fn handle_keyboard_input(&mut self, keycode: winit::keyboard::KeyCode, pressed: bool) {
        self.input_handler.handle_keyboard_input_event(keycode, pressed);
    }

    pub fn handle_window_focus(&mut self, focused: bool, window: Option<&Window>) {
        self.input_handler.handle_window_focus(focused, window);
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let delta_pos = winit::dpi::PhysicalPosition::new(delta.0, delta.1);
            self.handle_mouse_motion(delta_pos);
        }
    }

    pub fn get_position(&self) -> glam::Vec3 {
        self.camera.position
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.camera.position = position;
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn set_movement_speed(&mut self, speed: f32) {
        self.movement_speed = speed;
    }

    pub fn set_mouse_sensitivity(&mut self, sensitivity: f32) {
        self.mouse_sensitivity = sensitivity;
    }
} 