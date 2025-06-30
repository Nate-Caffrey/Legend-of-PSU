//! Window management implementation.

use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use log::error;

pub struct WindowManager {
    pub window: Option<Window>,
    pub size: Option<winit::dpi::PhysicalSize<u32>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            window: None,
            size: None,
        }
    }

    pub fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let window = event_loop.create_window(Window::default_attributes())
            .map_err(|e| {
                error!("Failed to create window: {:?}", e);
                e
            })?;
        
        let size = window.inner_size();
        self.size = Some(size);
        self.window = Some(window);
        Ok(())
    }

    pub fn set_window(&mut self, window: Window) {
        let size = window.inner_size();
        self.size = Some(size);
        self.window = Some(window);
    }

    pub fn set_window_size(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = Some(size);
    }

    pub fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
        on_close: impl FnOnce(),
        on_resize: impl FnOnce(winit::dpi::PhysicalSize<u32>),
        on_redraw: impl FnOnce(),
        on_keyboard: impl FnOnce(winit::keyboard::KeyCode, bool),
        on_focus: impl FnOnce(bool),
    ) {
        match event {
            WindowEvent::CloseRequested => {
                on_close();
            },
            WindowEvent::RedrawRequested => {
                on_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                on_resize(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                    let pressed = event.state == winit::event::ElementState::Pressed;
                    on_keyboard(keycode, pressed);
                }
            }
            WindowEvent::Focused(focused) => {
                on_focus(focused);
            }
            _ => (),
        }
    }

    pub fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub fn get_window(&self) -> Option<&Window> {
        self.window.as_ref()
    }

    pub fn get_size(&self) -> Option<winit::dpi::PhysicalSize<u32>> {
        self.size
    }
} 