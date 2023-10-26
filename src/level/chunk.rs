use bevy::prelude::Component;

use crate::block_registry::BlockId;

#[derive(Component)]
pub struct Dirty;

pub const CHUNK_SIZE: usize = 32;

#[derive(Clone)]
pub struct Chunk {
    blocks: Vec<Option<BlockId>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl Chunk {
    pub fn block(&self, x: usize, y: usize, z: usize) -> &Option<BlockId> {
        &self.blocks[Self::index(x, y, z)]
    }

    pub fn block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Option<BlockId> {
        &mut self.blocks[Self::index(x, y, z)]
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}
