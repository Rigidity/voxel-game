use crate::model::{CubeModel, Model};

use super::Block;

#[derive(Clone)]
pub struct DirtBlock;

impl Block for DirtBlock {
    fn model(&self) -> &dyn Model {
        &CubeModel
    }
}
