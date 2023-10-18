use bevy::{prelude::Resource, utils::HashMap};

use crate::{
    chunk::{Chunk, CHUNK_SIZE},
    position::{BlockPos, ChunkPos},
};

#[derive(Default, Resource)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
}

impl Level {
    pub fn load_chunk(&mut self, position: &ChunkPos) -> &Chunk {
        if !self.is_loaded(position) {
            let chunk = self.generate_chunk(position);
            self.loaded_chunks.insert(position.clone(), chunk);
        }
        &self.loaded_chunks[position]
    }

    pub fn unload_chunk(&mut self, position: &ChunkPos) {
        self.loaded_chunks.remove(position);
    }

    pub fn is_loaded(&self, position: &ChunkPos) -> bool {
        self.loaded_chunks.contains_key(position)
    }

    pub fn block(&self, position: &BlockPos) -> Option<bool> {
        let (chunk_pos, (x, y, z)) = position.chunk_pos();

        self.loaded_chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.block_relative(x, y, z))
    }

    fn generate_chunk(&mut self, chunk_pos: &ChunkPos) -> Chunk {
        let mut chunk = Chunk::default();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block_pos = BlockPos::from(chunk_pos.clone()) + BlockPos::new(x, y, z);
                    if self.generate_block(&block_pos) {
                        *chunk.block_relative_mut(x, y, z) = true;
                    }
                }
            }
        }
        chunk
    }

    fn generate_block(&self, block_pos: &BlockPos) -> bool {
        block_pos.y <= 64
    }
}
