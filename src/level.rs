use bevy::{prelude::Resource, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    chunk::{Chunk, CHUNK_SIZE},
    position::ChunkPos,
};

#[derive(Default, Resource)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
    noise: Perlin,
}

impl Level {
    pub fn load_chunk(&mut self, position: &ChunkPos) -> &Chunk {
        if self.get_chunk(position).is_none() {
            let chunk = self.generate_chunk(position);
            self.loaded_chunks.insert(position.clone(), chunk);
        }
        &self.loaded_chunks[position]
    }

    pub fn unload_chunk(&mut self, position: &ChunkPos) {
        self.loaded_chunks.remove(position);
    }

    pub fn get_chunk(&self, position: &ChunkPos) -> Option<&Chunk> {
        self.loaded_chunks.get(position)
    }

    fn generate_chunk(&mut self, chunk_pos: &ChunkPos) -> Chunk {
        let mut chunk = Chunk::default();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
                let block_z = chunk_pos.z * CHUNK_SIZE as i32 + z as i32;
                let noise = self
                    .noise
                    .get([block_x as f64 / 70.0, block_z as f64 / 70.0]);
                for y in 0..CHUNK_SIZE {
                    let block_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;
                    if block_y as f64 <= noise * 10.0 {
                        *chunk.block_relative_mut(x, y, z) = true;
                    }
                }
            }
        }
        chunk
    }
}
