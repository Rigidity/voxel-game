use bevy::prelude::Vec3;

use crate::level::{AdjacentBlocks, MeshBuilder};

pub mod dirt;

pub struct Block {
    pub render: fn(&mut MeshBuilder, AdjacentBlocks, Vec3),
}
