//! Application entry point.

use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // Initialize logging, panic hooks, etc.
    // TODO: Add proper error handling and logging setup
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Start the main app loop
    let mut app = game::App::default();
    let _ = event_loop.run_app(&mut app);
}