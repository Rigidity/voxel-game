use std::sync::Arc;

use bevy::prelude::{Deref, DerefMut};
use parking_lot::RwLock;

use super::chunk_data::ChunkData;

#[derive(Clone, Default, Deref, DerefMut)]
pub struct Chunk(Arc<RwLock<ChunkData>>);
