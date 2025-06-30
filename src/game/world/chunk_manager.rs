use std::collections::{HashMap, HashSet};
use glam::Vec3;
use crate::game::world::chunk::{Chunk, CHUNK_SIZE};
use crossbeam_channel::{Sender, Receiver, unbounded};

pub struct ChunkManager {
    pub loaded: HashMap<(i32, i32, i32), Chunk>,
    pub pending: HashSet<(i32, i32, i32)>,
    pub view_distance: i32,
    tx: Sender<(i32, i32, i32, Chunk)>,
    rx: Receiver<(i32, i32, i32, Chunk)>,
}

impl ChunkManager {
    pub fn new(view_distance: i32) -> Self {
        let (tx, rx) = unbounded();
        Self {
            loaded: HashMap::new(),
            pending: HashSet::new(),
            view_distance,
            tx,
            rx,
        }
    }

    pub fn update_chunks(&mut self, camera_pos: Vec3) {
        let cam_chunk = (
            (camera_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (camera_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (camera_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );
        // Request new chunks in view distance
        for dx in -self.view_distance..=self.view_distance {
            for dy in -self.view_distance..=self.view_distance {
                for dz in -self.view_distance..=self.view_distance {
                    let pos = (cam_chunk.0 + dx, cam_chunk.1 + dy, cam_chunk.2 + dz);
                    if !self.loaded.contains_key(&pos) && !self.pending.contains(&pos) {
                        let chunk_pos = Vec3::new(
                            pos.0 as f32 * CHUNK_SIZE as f32,
                            pos.1 as f32 * CHUNK_SIZE as f32,
                            pos.2 as f32 * CHUNK_SIZE as f32,
                        );
                        let tx = self.tx.clone();
                        self.pending.insert(pos);
                        std::thread::spawn(move || {
                            let chunk = Chunk::new(chunk_pos);
                            tx.send((pos.0, pos.1, pos.2, chunk)).ok();
                        });
                    }
                }
            }
        }
        // Unload distant chunks
        self.loaded.retain(|&(x, y, z), _| {
            (x - cam_chunk.0).abs() <= self.view_distance &&
            (y - cam_chunk.1).abs() <= self.view_distance &&
            (z - cam_chunk.2).abs() <= self.view_distance
        });
    }

    /// Call this every frame to receive finished chunks
    pub fn poll_new_chunks(&mut self, atlas_helper: &crate::engine::graphics::texture::AtlasUVHelper) {
        let mut to_remesh = Vec::new();
        while let Ok((x, y, z, mut chunk)) = self.rx.try_recv() {
            to_remesh.push(((x, y, z), chunk));
            self.pending.remove(&(x, y, z));
        }
        for ((x, y, z), mut chunk) in to_remesh {
            chunk.generate_mesh(self, atlas_helper);
            self.loaded.insert((x, y, z), chunk);
        }
    }

    pub fn all_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.loaded.values()
    }

    pub fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<crate::game::world::chunk::BlockType> {
        let chunk_x = (world_x as f32 / CHUNK_SIZE as f32).floor() as i32;
        let chunk_y = (world_y as f32 / CHUNK_SIZE as f32).floor() as i32;
        let chunk_z = (world_z as f32 / CHUNK_SIZE as f32).floor() as i32;
        let local_x = ((world_x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
        let local_y = ((world_y % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
        let local_z = ((world_z % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
        self.loaded.get(&(chunk_x, chunk_y, chunk_z)).map(|chunk| {
            chunk.blocks[local_x as usize][local_y as usize][local_z as usize]
        })
    }
} 