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

    pub fn deserialize(bytes: &[u8], registry: &BlockRegistry) -> Chunk {
        let mut chunk = Chunk::default();
        let mut i = 0;

        let name_list_len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
        i += 2;

        let mut names = Vec::with_capacity(name_list_len);
        for _ in 0..name_list_len {
            let name_len = bytes[i] as usize;
            i += 1;

            names.push(registry.block_id(&String::from_utf8_lossy(&bytes[i..i + name_len])));
            i += name_len;
        }

        let mut block = 0;

        while i < bytes.len() {
            let count = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
            i += 2;

            let index = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
            i += 2;

            for _ in 0..count {
                if index == 0 {
                    chunk.blocks[block] = None;
                    block += 1;
                } else {
                    chunk.blocks[block] = Some(names[index - 1]);
                    block += 1;
                }
            }
        }

        chunk
    }

    pub fn serialize(&self, registry: &BlockRegistry) -> Vec<u8> {
        let mut data = Vec::new();
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
                if let Some(index) = last {
                    data.extend((count as u16).to_be_bytes());
                    data.extend((index as u16).to_be_bytes());
                }
                last = Some(index);
                count = 1;
            }
        }

        if let Some(index) = last {
            data.extend((count as u16).to_be_bytes());
            data.extend((index as u16).to_be_bytes());
        }

        let mut bytes = Vec::new();
        bytes.extend((names.len() as u16).to_be_bytes());

        for name in names {
            bytes.push(name.len().try_into().unwrap());
            bytes.extend(name.as_bytes());
        }

        bytes.extend(data);
        bytes
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}
