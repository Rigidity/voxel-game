use std::sync::Arc;

use bevy::{prelude::Resource, utils::HashMap};
use serde::de::DeserializeOwned;

use crate::block::BlockType;

pub type DeserializeFn = fn(&[u8]) -> Result<Box<dyn BlockType>, bincode::Error>;

pub struct BlockInfo {
    deserialize: DeserializeFn,
    default: Arc<dyn BlockType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

#[derive(Resource, Default)]
pub struct BlockRegistry {
    block_types: HashMap<String, BlockId>,
    block_ids: Vec<BlockInfo>,
}

impl BlockRegistry {
    pub fn register<T>(&mut self, name: String)
    where
        T: BlockType + DeserializeOwned + Default + 'static,
    {
        let id = BlockId(self.block_types.len());
        self.block_ids.push(BlockInfo {
            deserialize: |bytes| {
                let lamp: T = bincode::deserialize(bytes)?;
                Ok(Box::new(lamp))
            },
            default: Arc::new(T::default()),
        });
        self.block_types.insert(name, id);
    }

    pub fn id(&self, name: &str) -> BlockId {
        self.block_types[name]
    }

    pub fn ids(&self) -> Vec<Arc<dyn BlockType>> {
        self.block_ids
            .iter()
            .map(|info| info.default.clone())
            .collect()
    }
}
