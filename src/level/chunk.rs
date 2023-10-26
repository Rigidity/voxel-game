use bevy::prelude::Component;
use indexmap::IndexSet;

use crate::block_registry::{BlockId, BlockRegistry};

#[derive(Component)]
pub struct Dirty;

pub const CHUNK_SIZE: usize = 32;

#[derive(Clone)]
pub struct Chunk {
    blocks: Vec<Option<BlockId>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl Chunk {
    pub fn block(&self, x: usize, y: usize, z: usize) -> &Option<BlockId> {
        &self.blocks[Self::index(x, y, z)]
    }

    pub fn block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Option<BlockId> {
        &mut self.blocks[Self::index(x, y, z)]
    }

    pub fn serialize(&self, registry: &BlockRegistry) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut names = IndexSet::new();
        let mut last = None;
        let mut count = 0;

        for block in self.blocks.iter() {
            let index = block
                .map(|id| names.insert_full(registry.name(id)).0 + 1)
                .unwrap_or_default();

            if last == Some(index) {
                count += 1;
            } else {
                last = Some(index);
                count = 0;
                bytes.extend((count as u16).to_be_bytes());
                bytes.extend((index as u16).to_be_bytes());
            }
        }

        if let Some(index) = last {
            bytes.extend((count as u16).to_be_bytes());
            bytes.extend((index as u16).to_be_bytes());
        }

        for name in names {
            bytes.push(name.len().try_into().unwrap());
            bytes.extend(name.as_bytes());
        }

        bytes
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}
