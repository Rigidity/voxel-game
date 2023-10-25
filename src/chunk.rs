use std::array;

use bevy::prelude::Component;

use crate::block::Block;

#[derive(Component)]
pub struct Dirty;

pub const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: array::from_fn(|_| array::from_fn(|_| array::from_fn(|_| Block::Empty))),
        }
    }
}

impl Chunk {
    pub fn block_relative(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[x][y][z]
    }

    pub fn block_relative_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[x][y][z]
    }
}
