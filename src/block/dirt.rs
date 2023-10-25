use bevy::prelude::Vec3;
use serde::{Deserialize, Serialize};

use crate::chunk_builder::{AdjacentBlocks, ChunkBuilder};

use super::BlockType;

#[derive(Default, Serialize, Deserialize)]
pub struct DirtBlock;

impl BlockType for DirtBlock {
    fn render(&self, chunk: &mut ChunkBuilder, adjacent: AdjacentBlocks, Vec3 { x, y, z }: Vec3) {
        // Left
        if !adjacent.left {
            let a = chunk.vertex([x, y, z], [-1.0, 0.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x, y + 1.0, z], [-1.0, 0.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x, y + 1.0, z + 1.0], [-1.0, 0.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x, y, z + 1.0], [-1.0, 0.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }

        // Right
        if !adjacent.right {
            let a = chunk.vertex([x + 1.0, y, z], [1.0, 0.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 1.0, y + 1.0, z], [1.0, 0.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 0.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x + 1.0, y, z + 1.0], [1.0, 0.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Top
        if !adjacent.top {
            let a = chunk.vertex([x, y + 1.0, z], [0.0, 1.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 1.0, y + 1.0, z], [0.0, 1.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 1.0, y + 1.0, z + 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x, y + 1.0, z + 1.0], [0.0, 1.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }

        // Bottom
        if !adjacent.bottom {
            let a = chunk.vertex([x, y, z], [0.0, -1.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 1.0, y, z], [0.0, -1.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 1.0, y, z + 1.0], [0.0, -1.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x, y, z + 1.0], [0.0, -1.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Front
        if !adjacent.front {
            let a = chunk.vertex([x, y, z + 1.0], [0.0, 0.0, 1.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 1.0, y, z + 1.0], [0.0, 0.0, 1.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 1.0, y + 1.0, z + 1.0], [0.0, 0.0, 1.0], [1.0, 1.0]);
            let d = chunk.vertex([x, y + 1.0, z + 1.0], [0.0, 0.0, 1.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Back
        if !adjacent.back {
            let a = chunk.vertex([x, y, z], [0.0, 0.0, -1.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 1.0, y, z], [0.0, 0.0, -1.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 1.0, y + 1.0, z], [0.0, 0.0, -1.0], [1.0, 1.0]);
            let d = chunk.vertex([x, y + 1.0, z], [0.0, 0.0, -1.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }
    }
}
