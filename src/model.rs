mod cube;

use bevy::prelude::Vec3;
pub use cube::*;

use crate::{
    block::AdjacentBlocks,
    chunk_builder::ChunkBuilder,
    registry::{Id, Registry},
};

pub type ModelRegistry = Registry<dyn Model>;
pub type ModelId = Id<dyn Model>;

pub trait Model: Send + Sync {
    fn render(&self, chunk: &mut ChunkBuilder, adjacent: AdjacentBlocks, translation: Vec3);
}
