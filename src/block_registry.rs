use bevy::{prelude::Resource, utils::HashMap};
use serde::de::DeserializeOwned;

use crate::block::Block;

pub type BlockData = Box<dyn Block>;
pub type DeserializeFn = fn(&[u8]) -> Result<BlockData, bincode::Error>;
pub type DefaultFn = fn() -> BlockData;

pub struct BlockType {
    deserialize: DeserializeFn,
    default: DefaultFn,
}

impl BlockType {
    pub fn deserialize(&self, bytes: &[u8]) -> Result<BlockData, bincode::Error> {
        (self.deserialize)(bytes)
    }

    pub fn default(&self) -> BlockData {
        (self.default)()
    }
}

#[derive(Resource, Default)]
pub struct BlockRegistry {
    block_types: HashMap<String, BlockType>,
}

impl BlockRegistry {
    pub fn register<T>(&mut self, name: String)
    where
        T: Block + DeserializeOwned + Default + 'static,
    {
        self.block_types.insert(
            name,
            BlockType {
                deserialize: |bytes| {
                    let lamp: T = bincode::deserialize(bytes)?;
                    Ok(Box::new(lamp))
                },
                default: || Box::new(T::default()),
            },
        );
    }
}
