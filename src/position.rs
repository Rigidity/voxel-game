use bevy::prelude::{Component, Vec3};
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::chunk::CHUNK_SIZE;

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Neg,
)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const X: BlockPos = BlockPos { x: 1, y: 0, z: 0 };
    pub const Y: BlockPos = BlockPos { x: 0, y: 1, z: 0 };
    pub const Z: BlockPos = BlockPos { x: 0, y: 0, z: 1 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn chunk_pos(&self) -> (ChunkPos, (usize, usize, usize)) {
        let (chunk_x, block_x) = (self.x / CHUNK_SIZE, self.x.rem_euclid(CHUNK_SIZE));
        let (chunk_y, block_y) = (self.y / CHUNK_SIZE, self.y.rem_euclid(CHUNK_SIZE));
        let (chunk_z, block_z) = (self.z / CHUNK_SIZE, self.z.rem_euclid(CHUNK_SIZE));
        (
            ChunkPos::new(chunk_x, chunk_y, chunk_z),
            (block_x, block_y, block_z),
        )
    }
}

impl From<ChunkPos> for BlockPos {
    fn from(value: ChunkPos) -> Self {
        Self::new(
            value.x * CHUNK_SIZE,
            value.y * CHUNK_SIZE,
            value.z * CHUNK_SIZE,
        )
    }
}

impl From<BlockPos> for Vec3 {
    fn from(value: BlockPos) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[derive(
    Component,
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Neg,
)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const X: ChunkPos = ChunkPos { x: 1, y: 0, z: 0 };
    pub const Y: ChunkPos = ChunkPos { x: 0, y: 1, z: 0 };
    pub const Z: ChunkPos = ChunkPos { x: 0, y: 0, z: 1 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn center(&self) -> Vec3 {
        let block_pos = BlockPos::from(self.clone());
        let add = CHUNK_SIZE / 2;
        Vec3::new(
            (block_pos.x + add),
            (block_pos.y + add),
            (block_pos.z + add),
        )
    }
}

impl From<ChunkPos> for Vec3 {
    fn from(value: ChunkPos) -> Self {
        BlockPos::from(value).into()
    }
}
