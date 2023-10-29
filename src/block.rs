use bevy::prelude::Vec3;

use crate::level::{AdjacentBlocks, ChunkBuilder};

pub mod dirt;

pub struct Block {
    pub render: fn(&mut ChunkBuilder, AdjacentBlocks, Vec3),
}
