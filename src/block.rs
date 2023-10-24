mod dirt;

pub use dirt::*;

use crate::model::Model;

#[derive(Debug, Clone, Copy)]
pub struct AdjacentBlocks {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
}

pub trait Block: Send + Sync {
    fn model(&self) -> &dyn Model;
}
