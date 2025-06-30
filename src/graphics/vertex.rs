use wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub texture_index: u32,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32,
            },
        ];
        
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

// 3D cube vertices with texture coordinates for all 6 faces
pub const CUBE_VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], texture_index: 0 },
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], texture_index: 0 },
    
    // Back face
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], texture_index: 0 },
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], texture_index: 0 },
    
    // Left face
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], texture_index: 0 },
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 1.0], texture_index: 0 },
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], texture_index: 0 },
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0], texture_index: 0 },
    
    // Right face
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], texture_index: 0 },
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0], texture_index: 0 },
    
    // Top face
    Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 0.0], texture_index: 0 },
    Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0], texture_index: 0 },
    
    // Bottom face
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0], texture_index: 0 },
    Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 0.0], texture_index: 0 },
    Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 0.0], texture_index: 0 },
];

pub const CUBE_INDICES: &[u16] = &[
    // Front face
    0, 1, 2,  2, 3, 0,
    // Back face
    4, 5, 6,  6, 7, 4,
    // Left face
    8, 9, 10, 10, 11, 8,
    // Right face
    12, 13, 14, 14, 15, 12,
    // Top face
    16, 17, 18, 18, 19, 16,
    // Bottom face
    20, 21, 22, 22, 23, 20,
]; 