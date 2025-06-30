//! Game state management implementation.

use std::time::Instant;

pub struct GameState {
    pub show_fps: bool,
    pub last_fps_print: Instant,
    pub frame_count: u32,
    pub last_fps: u32,
    pub fullscreen: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            show_fps: false,
            last_fps_print: Instant::now(),
            frame_count: 0,
            last_fps: 0,
            fullscreen: false,
        }
    }

    pub fn update_frame_count(&mut self) {
        self.frame_count += 1;
    }

    pub fn update_fps_display(&mut self) -> Option<u32> {
        if !self.show_fps {
            return None;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_print);
        
        if elapsed.as_secs_f32() >= 1.0 {
            self.last_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_print = now;
            Some(self.last_fps)
        } else {
            None
        }
    }

    pub fn toggle_fps_display(&mut self) {
        self.show_fps = !self.show_fps;
        println!("Show FPS: {}", self.show_fps);
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    pub fn get_fps(&self) -> u32 {
        self.last_fps
    }

    pub fn is_fps_display_enabled(&self) -> bool {
        self.show_fps
    }

    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }
} 