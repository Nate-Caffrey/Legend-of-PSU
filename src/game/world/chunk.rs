use glam::Vec3;
use crate::engine::graphics::vertex::Vertex;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_F: f32 = CHUNK_SIZE as f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air)
    }
}

pub struct Chunk {
    pub position: Vec3,
    pub blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Chunk {
    pub fn new(position: Vec3) -> Self {
        let mut chunk = Self {
            position,
            blocks: [[[BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        
        chunk.generate_terrain();
        chunk.generate_mesh();
        
        chunk
    }

    fn value_noise(x: i32, z: i32, seed: u32) -> f32 {
        // Simple hash-based value noise
        let n = x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263)).wrapping_add(seed as i32 * 31);
        let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        ((n & 0x7fffffff) as f32) / 0x7fffffff as f32
    }

    pub fn generate_terrain(&mut self) {
        // Only generate terrain for ground chunks (y == 0)
        if self.position.y != 0.0 {
            return;
        }
        let seed = 42;
        let scale = 0.15;
        let min_height = 1;
        let max_height = CHUNK_SIZE as i32 / 4; // Lower hills
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let nx = self.position.x as i32 + x as i32;
                let nz = self.position.z as i32 + z as i32;
                let noise = Self::value_noise((nx as f32 * scale) as i32, (nz as f32 * scale) as i32, seed);
                let height = min_height + ((noise * (max_height - min_height) as f32).round() as i32);
                for y in 0..height.clamp(0, CHUNK_SIZE as i32 - 1) {
                    let block = if y == height - 1 {
                        BlockType::Grass
                    } else if y > height - 5 {
                        BlockType::Dirt
                    } else {
                        BlockType::Stone
                    };
                    self.blocks[x][y as usize][z] = block;
                }
            }
        }
    }

    pub fn generate_mesh(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        
        let mut vertex_offset = 0;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks[x][y][z].is_solid() {
                        self.add_block_mesh(x, y, z, &mut vertex_offset);
                    }
                }
            }
        }
    }

    fn add_block_mesh(&mut self, x: usize, y: usize, z: usize, _vertex_offset: &mut u16) {
        let world_x = self.position.x + x as f32;
        let world_y = self.position.y + y as f32;
        let world_z = self.position.z + z as f32;

        // Each face: (neighbor offset, 4 positions, face index)
        let faces = [
            // Front (+z)
            ((0, 0, 1), [
                [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]
            ], 0),
            // Back (-z)
            ((0, 0, -1), [
                [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]
            ], 1),
            // Left (-x)
            ((-1, 0, 0), [
                [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]
            ], 2),
            // Right (+x)
            ((1, 0, 0), [
                [1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]
            ], 3),
            // Top (+y)
            ((0, 1, 0), [
                [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]
            ], 4),
            // Bottom (-y)
            ((0, -1, 0), [
                [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]
            ], 5),
        ];
        let face_uvs = [
            [0.0, 1.0], // bottom-left
            [1.0, 1.0], // bottom-right
            [1.0, 0.0], // top-right
            [0.0, 0.0], // top-left
        ];

        let block_type = self.blocks[x][y][z];

        for (face_idx, (offset, positions, _face_id)) in faces.iter().enumerate() {
            let nx = x as isize + offset.0;
            let ny = y as isize + offset.1;
            let nz = z as isize + offset.2;
            let neighbor_solid = if nx >= 0 && nx < CHUNK_SIZE as isize && ny >= 0 && ny < CHUNK_SIZE as isize && nz >= 0 && nz < CHUNK_SIZE as isize {
                self.blocks[nx as usize][ny as usize][nz as usize].is_solid()
            } else {
                false
            };
            if !neighbor_solid {
                let base = self.vertices.len() as u16;
                // Determine texture index for this face
                let texture_index = match block_type {
                    BlockType::Grass => match face_idx {
                        4 => 0, // Top
                        5 => 2, // Bottom
                        _ => 1, // Sides
                    },
                    BlockType::Dirt => 2,
                    BlockType::Stone => 3,
                    BlockType::Air => 0, // Should not happen
                };
                for i in 0..4 {
                    self.vertices.push(crate::engine::graphics::vertex::Vertex {
                        position: [
                            world_x + positions[i][0],
                            world_y + positions[i][1],
                            world_z + positions[i][2],
                        ],
                        tex_coords: face_uvs[i],
                        texture_index,
                    });
                }
                self.indices.extend_from_slice(&[
                    base, base + 1, base + 2,
                    base, base + 2, base + 3,
                ]);
            }
        }
    }

    pub fn get_vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn get_index_count(&self) -> u32 {
        self.indices.len() as u32
    }
} 