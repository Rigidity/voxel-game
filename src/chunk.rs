use bevy::prelude::Component;

#[derive(Component)]
pub struct Dirty;

pub const CHUNK_SIZE: i32 = 32;

pub struct Chunk {
    blocks: [[[bool; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: [[[false; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
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
