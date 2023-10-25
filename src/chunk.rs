use std::array;

use bevy::prelude::Component;

use crate::block_registry::BlockData;

#[derive(Component)]
pub struct Dirty;

pub const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    blocks: [[[Option<BlockData>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: array::from_fn(|_| array::from_fn(|_| array::from_fn(|_| None))),
        }
    }
}

impl Chunk {
    pub fn block_relative(&self, x: usize, y: usize, z: usize) -> &Option<BlockData> {
        &self.blocks[x][y][z]
    }

    pub fn block_relative_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Option<BlockData> {
        &mut self.blocks[x][y][z]
    }
}
