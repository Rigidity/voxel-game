use std::sync::{Arc, RwLock};

use bevy::{prelude::Resource, utils::HashMap};
use noise::Perlin;

use crate::{chunk::Chunk, position::ChunkPos};

#[derive(Default, Resource)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Arc<RwLock<Chunk>>>,
    noise: Perlin,
}

impl Level {
    pub fn is_loaded(&self, position: &ChunkPos) -> bool {
        self.loaded_chunks.contains_key(position)
    }

    pub fn add_chunk(&mut self, position: &ChunkPos, chunk: Arc<RwLock<Chunk>>) {
        self.loaded_chunks.insert(position.clone(), chunk);
    }

    pub fn remove_chunk(&mut self, position: &ChunkPos) {
        self.loaded_chunks.remove(position);
    }

    pub fn chunk(&self, position: &ChunkPos) -> Option<&Arc<RwLock<Chunk>>> {
        self.loaded_chunks.get(position)
    }

    pub fn noise(&self) -> &Perlin {
        &self.noise
    }
}
