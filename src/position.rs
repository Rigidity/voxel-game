use bevy::prelude::{Component, Vec3};
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::level::CHUNK_SIZE;

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
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

    pub fn block_in_chunk(self) -> (usize, usize, usize) {
        (
            self.x.rem_euclid(CHUNK_SIZE as i32) as usize,
            self.y.rem_euclid(CHUNK_SIZE as i32) as usize,
            self.z.rem_euclid(CHUNK_SIZE as i32) as usize,
        )
    }
}

fn div_floor(a: i32, b: i32) -> i32 {
    if a >= 0 || a % b == 0 {
        a / b
    } else {
        a / b - 1
    }
}

impl From<ChunkPos> for BlockPos {
    fn from(value: ChunkPos) -> Self {
        Self::new(
            value.x * CHUNK_SIZE as i32,
            value.y * CHUNK_SIZE as i32,
            value.z * CHUNK_SIZE as i32,
        )
    }
}

impl From<BlockPos> for Vec3 {
    fn from(value: BlockPos) -> Self {
        Self::new(value.x as f32, value.y as f32, value.z as f32)
    }
}

impl From<Vec3> for BlockPos {
    fn from(value: Vec3) -> Self {
        Self::new(value.x as i32, value.y as i32, value.z as i32)
    }
}

#[derive(
    Component,
    Default,
    Debug,
    Clone,
    Copy,
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

    pub fn center(self) -> Vec3 {
        let block_pos = BlockPos::from(self);
        let add = CHUNK_SIZE as i32 / 2;
        Vec3::new(
            (block_pos.x + add) as f32,
            (block_pos.y + add) as f32,
            (block_pos.z + add) as f32,
        )
    }

    pub fn is_adjacent(self, pos: ChunkPos) -> bool {
        let is_left = pos == self - ChunkPos::X;
        let is_right = pos == self + ChunkPos::X;
        let is_bottom = pos == self - ChunkPos::Y;
        let is_top = pos == self + ChunkPos::Y;
        let is_back = pos == self - ChunkPos::Z;
        let is_front = pos == self + ChunkPos::Z;
        is_left || is_right || is_bottom || is_top || is_back || is_front
    }
}

impl From<ChunkPos> for Vec3 {
    fn from(value: ChunkPos) -> Self {
        BlockPos::from(value).into()
    }
}

impl From<Vec3> for ChunkPos {
    fn from(value: Vec3) -> Self {
        BlockPos::from(value).into()
    }
}

impl From<BlockPos> for ChunkPos {
    fn from(value: BlockPos) -> Self {
        Self::new(
            div_floor(value.x, CHUNK_SIZE as i32),
            div_floor(value.y, CHUNK_SIZE as i32),
            div_floor(value.z, CHUNK_SIZE as i32),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_pos() {
        macro_rules! test_rem {
            ($block_pos:expr => $chunk_pos:expr, $remainder:expr) => {
                let block_pos = $block_pos;
                let chunk_pos = ChunkPos::from(block_pos);
                let remainder = block_pos.block_in_chunk();
                assert_eq!(chunk_pos, $chunk_pos);
                assert_eq!(remainder, $remainder);
            };
        }

        test_rem!(BlockPos::new(0, 0, 0) => ChunkPos::new(0, 0, 0), (0, 0, 0));
        test_rem!(BlockPos::new(5, 0, 0) => ChunkPos::new(0, 0, 0), (5, 0, 0));
        test_rem!(BlockPos::new(31, 0, 0) => ChunkPos::new(0, 0, 0), (31, 0, 0));
        test_rem!(BlockPos::new(32, 0, 0) => ChunkPos::new(1, 0, 0), (0, 0, 0));
        test_rem!(BlockPos::new(-1, 0, 0) => ChunkPos::new(-1, 0, 0), (31, 0, 0));
    }
}
