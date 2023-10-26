use bevy::{prelude::Resource, utils::HashMap};
use noise::Perlin;

use crate::position::ChunkPos;

mod chunk;
mod chunk_builder;
mod level_gen;

pub use chunk::*;
pub use chunk_builder::*;
pub use level_gen::*;

#[derive(Default, Resource)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
    noise: Perlin,
}

impl Level {
    pub fn is_loaded(&self, position: &ChunkPos) -> bool {
        self.loaded_chunks.contains_key(position)
    }

    pub fn add_chunk(&mut self, position: &ChunkPos, chunk: Chunk) {
        self.loaded_chunks.insert(position.clone(), chunk);
    }

    pub fn remove_chunk(&mut self, position: &ChunkPos) {
        self.loaded_chunks.remove(position);
    }

    pub fn chunk(&self, position: &ChunkPos) -> Option<&Chunk> {
        self.loaded_chunks.get(position)
    }

    pub fn chunk_mut(&mut self, position: &ChunkPos) -> Option<&mut Chunk> {
        self.loaded_chunks.get_mut(position)
    }

    pub fn noise(&self) -> Perlin {
        self.noise
    }
}
