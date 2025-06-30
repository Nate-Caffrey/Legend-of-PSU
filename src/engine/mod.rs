//! Core engine code, reusable across different games or projects.

pub mod graphics;
pub mod input;
pub mod shaders;
pub mod window;

// Re-export commonly used types
pub use graphics::{renderer::Renderer, texture::Texture, vertex::Vertex}; 