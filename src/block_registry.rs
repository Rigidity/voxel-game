use std::{
    num::NonZeroU16,
    sync::{Arc, RwLock},
};

use bevy::{prelude::*, utils::HashMap};

use crate::block::Block;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(NonZeroU16);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SharedBlockRegistry(Arc<RwLock<BlockRegistry>>);

#[derive(Default)]
pub struct BlockRegistry {
    names: HashMap<String, BlockId>,
    blocks: Vec<Block>,
}

impl BlockRegistry {
    pub fn register(&mut self, name: String, block: Block) {
        let len: u16 = self.blocks.len().try_into().unwrap();
        let id = BlockId(NonZeroU16::new(len + 1).unwrap());
        self.blocks.push(block);
        self.names.insert(name, id);
    }

    pub fn block_id(&self, name: &str) -> BlockId {
        self.names[name]
    }

    pub fn name(&self, id: BlockId) -> &str {
        self.names
            .iter()
            .find(|item| *item.1 == id)
            .map(|item| item.0.as_str())
            .unwrap()
    }

    pub fn block(&self, id: BlockId) -> &Block {
        &self.blocks[id.0.get() as usize - 1]
    }
}
