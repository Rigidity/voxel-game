mod dirt;

use bevy::prelude::Vec3;
pub use dirt::*;
use erased_serde::serialize_trait_object;

use crate::chunk_builder::{AdjacentBlocks, ChunkBuilder};

pub trait Block: erased_serde::Serialize + Send + Sync {
    fn render(&self, chunk: &mut ChunkBuilder, adjacent: AdjacentBlocks, translation: Vec3);
}

serialize_trait_object!(Block);
