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

    pub fn generate_mesh(&mut self, chunk_manager: &crate::game::world::chunk_manager::ChunkManager) {
        self.vertices.clear();
        self.indices.clear();
        let mut vertex_offset = 0;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks[x][y][z].is_solid() {
                        self.add_block_mesh(x, y, z, &mut vertex_offset, chunk_manager);
                    }
                }
            }
        }
    }

    fn add_block_mesh(&mut self, x: usize, y: usize, z: usize, _vertex_offset: &mut u16, chunk_manager: &crate::game::world::chunk_manager::ChunkManager) {
        let world_x = self.position.x as i32 + x as i32;
        let world_y = self.position.y as i32 + y as i32;
        let world_z = self.position.z as i32 + z as i32;
        let faces = [
            ((0, 0, 1), [
                [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]
            ], 0),
            ((0, 0, -1), [
                [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]
            ], 1),
            ((-1, 0, 0), [
                [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]
            ], 2),
            ((1, 0, 0), [
                [1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]
            ], 3),
            ((0, 1, 0), [
                [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]
            ], 4),
            ((0, -1, 0), [
                [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]
            ], 5),
        ];
        let face_uvs = [
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
        ];
        let block_type = self.blocks[x][y][z];
        for (face_idx, (offset, positions, _face_id)) in faces.iter().enumerate() {
            let nx = world_x + offset.0;
            let ny = world_y + offset.1;
            let nz = world_z + offset.2;
            let neighbor_solid = chunk_manager.get_block(nx, ny, nz).map_or(false, |b| b.is_solid());
            if !neighbor_solid {
                let base = self.vertices.len() as u16;
                let texture_index = match block_type {
                    crate::game::world::chunk::BlockType::Grass => match face_idx {
                        4 => 0,
                        5 => 2,
                        _ => 1,
                    },
                    crate::game::world::chunk::BlockType::Dirt => 2,
                    crate::game::world::chunk::BlockType::Stone => 3,
                    crate::game::world::chunk::BlockType::Air => 0,
                };
                for i in 0..4 {
                    self.vertices.push(crate::engine::graphics::vertex::Vertex {
                        position: [
                            self.position.x + x as f32 + positions[i][0],
                            self.position.y + y as f32 + positions[i][1],
                            self.position.z + z as f32 + positions[i][2],
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