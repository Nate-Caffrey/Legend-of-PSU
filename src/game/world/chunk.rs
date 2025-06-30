use glam::Vec3;
use crate::engine::graphics::vertex::{BlockFaceInstance};
use wgpu::util::DeviceExt;
use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_F: f32 = CHUNK_SIZE as f32;
pub const OCCLUSION_DISTANCE_CHUNKS: f32 = 3.0;

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
    pub block_face_instances: Vec<BlockFaceInstance>,
    pub instance_buffer: Option<wgpu::Buffer>,
}

impl Chunk {
    pub fn new(position: Vec3) -> Self {
        let mut chunk = Self {
            position,
            blocks: [[[BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            block_face_instances: Vec::new(),
            instance_buffer: None,
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
        self.block_face_instances.clear();
        let mut visible_air = vec![vec![vec![false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let mut queue = VecDeque::new();
        // Enqueue all boundary air blocks
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let is_boundary = x == 0 || y == 0 || z == 0 || x == CHUNK_SIZE - 1 || y == CHUNK_SIZE - 1 || z == CHUNK_SIZE - 1;
                    if is_boundary && !self.blocks[x][y][z].is_solid() {
                        visible_air[x][y][z] = true;
                        queue.push_back((x, y, z));
                    }
                }
            }
        }
        // Flood fill from boundary air
        let neighbors = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];
        while let Some((x, y, z)) = queue.pop_front() {
            for (dx, dy, dz) in neighbors.iter() {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                let nz = z as isize + dz;
                if nx >= 0 && ny >= 0 && nz >= 0 && nx < CHUNK_SIZE as isize && ny < CHUNK_SIZE as isize && nz < CHUNK_SIZE as isize {
                    let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                    if !self.blocks[nx][ny][nz].is_solid() && !visible_air[nx][ny][nz] {
                        visible_air[nx][ny][nz] = true;
                        queue.push_back((nx, ny, nz));
                    }
                }
            }
        }
        // Only add faces adjacent to visible air
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks[x][y][z].is_solid() {
                        for (face_idx, offset) in [
                            (0, 0, 1),   // Front
                            (0, 0, -1),  // Back
                            (-1, 0, 0),  // Left
                            (1, 0, 0),   // Right
                            (0, 1, 0),   // Top
                            (0, -1, 0),  // Bottom
                        ].iter().enumerate() {
                            let nx = x as isize + offset.0;
                            let ny = y as isize + offset.1;
                            let nz = z as isize + offset.2;
                            let mut air_visible = false;
                            if nx >= 0 && ny >= 0 && nz >= 0 && nx < CHUNK_SIZE as isize && ny < CHUNK_SIZE as isize && nz < CHUNK_SIZE as isize {
                                let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
                                air_visible = visible_air[nx][ny][nz];
                            } else {
                                // At chunk boundary, check neighbor chunk
                                let world_x = self.position.x as i32 + x as i32 + offset.0 as i32;
                                let world_y = self.position.y as i32 + y as i32 + offset.1 as i32;
                                let world_z = self.position.z as i32 + z as i32 + offset.2 as i32;
                                air_visible = chunk_manager.get_block(world_x, world_y, world_z).map_or(true, |b| !b.is_solid());
                            }
                            if air_visible {
                                self.block_face_instances.push(BlockFaceInstance {
                                    position: [self.position.x + x as f32, self.position.y + y as f32, self.position.z + z as f32],
                                    face: face_idx as u32,
                                    block_type: match self.blocks[x][y][z] {
                                        crate::game::world::chunk::BlockType::Grass => 0,
                                        crate::game::world::chunk::BlockType::Dirt => 1,
                                        crate::game::world::chunk::BlockType::Stone => 2,
                                        crate::game::world::chunk::BlockType::Air => 255,
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn build_instance_buffer(&mut self, device: &wgpu::Device) {
        if self.block_face_instances.is_empty() {
            self.instance_buffer = None;
        } else {
            self.instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Instance Buffer"),
                contents: bytemuck::cast_slice(&self.block_face_instances),
                usage: wgpu::BufferUsages::VERTEX,
            }));
        }
    }

    /// Returns true if there is a clear line of sight from camera_pos to face_center (no solid blocks in between)
    pub fn is_face_visible_from_camera(
        camera_pos: Vec3,
        face_center: Vec3,
        chunk_manager: &crate::game::world::chunk_manager::ChunkManager,
    ) -> bool {
        let dir = (face_center - camera_pos).normalize();
        let dist = (face_center - camera_pos).length();
        let steps = (dist * 2.0) as usize; // 0.5 block steps
        for i in 1..steps {
            let p = camera_pos + dir * (i as f32 * 0.5);
            let wx = p.x.floor() as i32;
            let wy = p.y.floor() as i32;
            let wz = p.z.floor() as i32;
            if let Some(block) = chunk_manager.get_block(wx, wy, wz) {
                if block.is_solid() {
                    return false;
                }
            }
        }
        true
    }
} 