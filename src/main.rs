//! Application entry point.

use winit::event_loop::{ControlFlow, EventLoop};
use log::{info, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Logger initialized");

    // Initialize logging, panic hooks, etc.
    // TODO: Add proper error handling and logging setup
    
    let event_loop = EventLoop::new().map_err(|e| {
        error!("Failed to create event loop: {:?}", e);
        e
    })?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Start the main app loop
    let mut app = game::App::default();
    if let Err(e) = event_loop.run_app(&mut app) {
        error!("Application error: {:?}", e);
        return Err(Box::new(e));
    }

    Ok(())
}