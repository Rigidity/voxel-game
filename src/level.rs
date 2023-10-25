use std::sync::{Arc, RwLock};

use bevy::{prelude::Resource, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    block::DirtBlock,
    chunk::{Chunk, CHUNK_SIZE},
    position::ChunkPos,
};

#[derive(Default, Resource)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Arc<RwLock<Chunk>>>,
    noise: Perlin,
}

impl Level {
    pub fn load_chunk(&mut self, position: &ChunkPos) -> &Arc<RwLock<Chunk>> {
        if self.chunk(position).is_none() {
            let chunk = self.generate_chunk(position);
            self.loaded_chunks
                .insert(position.clone(), Arc::new(RwLock::new(chunk)));
        }
        &self.loaded_chunks[position]
    }

    pub fn unload_chunk(&mut self, position: &ChunkPos) {
        self.loaded_chunks.remove(position);
    }

    pub fn chunk(&self, position: &ChunkPos) -> Option<&Arc<RwLock<Chunk>>> {
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
                    .get([block_x as f64 / 90.0, block_z as f64 / 90.0]);
                for y in 0..CHUNK_SIZE {
                    let block_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;
                    if block_y as f64 <= noise * 18.0 {
                        *chunk.block_relative_mut(x, y, z) = Some(Box::new(DirtBlock));
                    }
                }
            }
        }
        chunk
    }
}
