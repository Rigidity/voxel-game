use itertools::Itertools;
use noise::{NoiseFn, Perlin};

use crate::{block_registry::BlockRegistry, position::ChunkPos};

use super::chunk_data::{ChunkData, CHUNK_SIZE};

pub fn generate_chunk(noise: &Perlin, registry: &BlockRegistry, pos: ChunkPos) -> ChunkData {
    let mut chunk = ChunkData::default();
    let dirt = registry.read().block_id("dirt");

    for (x, z) in (0..CHUNK_SIZE).cartesian_product(0..CHUNK_SIZE) {
        let block_x = pos.x * CHUNK_SIZE as i32 + x as i32;
        let block_z = pos.z * CHUNK_SIZE as i32 + z as i32;
        let noise = noise.get([block_x as f64 / 90.0, block_z as f64 / 90.0]);
        for y in 0..CHUNK_SIZE {
            let block_y = pos.y * CHUNK_SIZE as i32 + y as i32;
            if block_y as f64 <= noise * 18.0 {
                *chunk.block_mut(x, y, z) = Some(dirt);
            }
        }
    }

    chunk
}
