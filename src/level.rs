use bevy::{prelude::Resource, utils::HashMap};
use noise::{NoiseFn, Simplex};

use crate::{
    chunk::{Chunk, CHUNK_SIZE},
    position::{BlockPos, ChunkPos},
};

#[derive(Default, Resource)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
    noise: Simplex,
}

impl Level {
    pub fn load_chunk(&mut self, position: &ChunkPos) -> &Chunk {
        if !self.loaded_chunks.contains_key(position) {
            let chunk = self.generate_chunk(position);
            self.loaded_chunks.insert(position.clone(), chunk);
        }
        &self.loaded_chunks[position]
    }

    pub fn block(&self, position: &BlockPos) -> bool {
        let (chunk_pos, (x, y, z)) = position.chunk_pos();

        if let Some(chunk) = self.loaded_chunks.get(&chunk_pos) {
            chunk.block_relative(x, y, z)
        } else {
            self.generate_block(position)
        }
    }

    fn generate_chunk(&mut self, chunk_pos: &ChunkPos) -> Chunk {
        let mut chunk = Chunk::default();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block_pos = BlockPos::from(chunk_pos.clone()) + BlockPos::new(x, y, z);
                    if self.generate_block(&block_pos) {
                        *chunk.block_relative_mut(x as usize, y as usize, z as usize) = true;
                    }
                }
            }
        }
        chunk
    }

    fn generate_block(&self, block_pos: &BlockPos) -> bool {
        let noise = self
            .noise
            .get([block_pos.x as f64 / 100.0, block_pos.z as f64 / 100.0]);
        let height = noise * 16.0 + 64.0;
        block_pos.y as f64 <= height
    }
}
