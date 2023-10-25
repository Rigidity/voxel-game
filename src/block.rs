mod dirt;

use bevy::prelude::Vec3;
pub use dirt::*;
use erased_serde::serialize_trait_object;

use crate::{
    block_registry::BlockId,
    chunk_builder::{AdjacentBlocks, ChunkBuilder},
};

pub trait BlockType: erased_serde::Serialize + Send + Sync {
    fn render(&self, chunk: &mut ChunkBuilder, adjacent: AdjacentBlocks, translation: Vec3);
}

serialize_trait_object!(BlockType);

pub enum Block {
    Empty,
    Id(BlockId),
    Data(Box<dyn BlockType>),
}

impl Block {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}
