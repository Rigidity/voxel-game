use bevy::prelude::Component;

#[derive(Component)]
pub struct Dirty;

pub const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    blocks: [[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

impl Chunk {
    pub fn block_relative(&self, x: usize, y: usize, z: usize) -> bool {
        self.blocks[x][y][z]
    }

    pub fn block_relative_mut(&mut self, x: usize, y: usize, z: usize) -> &mut bool {
        &mut self.blocks[x][y][z]
    }
}
