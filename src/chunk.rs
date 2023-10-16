use bevy::prelude::Component;

use crate::block::BlockPos;

#[derive(Component)]
pub struct Dirty;

#[derive(Default)]
pub struct Chunk {
    blocks: [[[bool; 16]; 16]; 16],
}

impl Chunk {
    pub fn set_block(&mut self, position: BlockPos, value: bool) {
        self.blocks[position.x() as usize][position.y() as usize][position.z() as usize] = value;
    }

    pub fn get_block(&self, position: BlockPos) -> bool {
        self.blocks[position.x() as usize][position.y() as usize][position.z() as usize]
    }
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(i32, i32, i32);

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(x, y, z)
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }

    pub fn z(&self) -> i32 {
        self.2
    }
}
