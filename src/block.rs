use bevy::prelude::Vec3;

pub mod dirt;

pub struct Block {
    pub render: fn(&mut ChunkBuilder, AdjacentBlocks, Vec3),
}
