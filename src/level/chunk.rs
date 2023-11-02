use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::RwLock;

use super::chunk_data::ChunkData;

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct Chunk(Arc<RwLock<ChunkData>>);

impl Chunk {
    pub fn new(data: ChunkData) -> Self {
        Self(Arc::new(RwLock::new(data)))
    }
}
