use bevy::{prelude::Resource, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    block::BlockPos,
    chunk::{Chunk, ChunkPos},
};

#[derive(Resource, Default)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
    perlin_noise: Perlin,
}

impl Level {
    pub fn load_chunk(&mut self, position: ChunkPos) -> &Chunk {
        if !self.loaded_chunks.contains_key(&position) {
            let chunk = self.create_chunk(position);
            self.loaded_chunks.insert(position, chunk);
        }
        &self.loaded_chunks[&position]
    }

    fn create_chunk(&mut self, position: ChunkPos) -> Chunk {
        let mut chunk = Chunk::default();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let total_x = position.x() * 16 + x as i32;
                    let total_y = position.y() * 16 + y as i32;
                    let total_z = position.z() * 16 + z as i32;

                    let noise = self
                        .perlin_noise
                        .get([total_x as f64 / 16.0, total_z as f64 / 16.0]);

                    let height = noise * 12.0 + 16.0;

                    if total_y as f64 <= height {
                        chunk.set_block(BlockPos::new(x, y, z), true);
                    }
                }
            }
        }
        chunk
    }
}
