use std::collections::HashSet;
use winit::keyboard::KeyCode;
use winit::window::{Window, Fullscreen, CursorGrabMode};
use log::debug;

use crate::game::world::camera::Camera;
use crate::game::world::chunk_manager::ChunkManager;

pub struct InputHandler {
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,
    pressed_keys: HashSet<KeyCode>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
            movement_speed: 0.1,
            pressed_keys: HashSet::new(),
        }
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_keyboard_input_event(
        &mut self,
        keycode: KeyCode,
        pressed: bool,
    ) {
        if pressed {
            self.pressed_keys.insert(keycode);
        } else {
            self.pressed_keys.remove(&keycode);
        }
    }

    pub fn apply_movement(&self, camera: &mut Camera) {
        use KeyCode::*;
        let mut direction = glam::Vec3::ZERO;
        let yaw = camera.yaw;
        let forward = glam::Vec3::new(yaw.sin(), 0.0, -yaw.cos());
        let right = glam::Vec3::new(yaw.cos(), 0.0, yaw.sin());

        if self.pressed_keys.contains(&KeyW) {
            direction += right;
        }
        if self.pressed_keys.contains(&KeyS) {
            direction -= right;
        }
        if self.pressed_keys.contains(&KeyA) {
            direction += forward;
        }
        if self.pressed_keys.contains(&KeyD) {
            direction -= forward;
        }
        if self.pressed_keys.contains(&KeyCode::Space) {
            direction.y += 1.0;
        }
        if self.pressed_keys.contains(&KeyCode::ShiftLeft) || self.pressed_keys.contains(&KeyCode::ShiftRight) {
            direction.y -= 1.0;
        }

        if direction != glam::Vec3::ZERO {
            let norm = direction.normalize();
            camera.position += norm * self.movement_speed;
            debug!("Camera moved: {:?}", camera.position);
        }
    }

    pub fn handle_mouse_motion(&self, delta: (f64, f64), camera: &mut Camera) {
        let (delta_x, delta_y) = delta;
        camera.rotate(
            delta_x as f32 * self.mouse_sensitivity,
            -delta_y as f32 * self.mouse_sensitivity,
        );
    }

    pub fn handle_window_focus(&self, focused: bool, window: Option<&Window>) {
        if let Some(window) = window {
            if focused {
                let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                window.set_cursor_visible(false);
                debug!("Window focused, cursor locked and hidden");
            } else {
                let _ = window.set_cursor_grab(CursorGrabMode::None);
                window.set_cursor_visible(true);
                debug!("Window unfocused, cursor unlocked and visible");
            }
        }
    }

    pub fn handle_fullscreen_toggle(&mut self, fullscreen: &mut bool, window: Option<&Window>) {
        if let Some(window) = window {
            if *fullscreen {
                window.set_fullscreen(None);
                debug!("Exited fullscreen mode");
            } else {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                debug!("Entered fullscreen mode");
            }
            *fullscreen = !*fullscreen;
        }
    }

    pub fn set_mouse_sensitivity(&mut self, sensitivity: f32) {
        self.mouse_sensitivity = sensitivity;
    }

    pub fn set_movement_speed(&mut self, speed: f32) {
        self.movement_speed = speed;
    }
} 