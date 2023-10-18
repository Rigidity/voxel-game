use bevy::prelude::Vec3;

use crate::chunk_builder::ChunkBuilder;

pub struct AdjacentBlocks {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
}

pub trait Block {
    fn render(chunk: &mut ChunkBuilder, adjacent: &AdjacentBlocks, translation: Vec3);
}

pub struct BasicBlock;

impl Block for BasicBlock {
    fn render(chunk: &mut ChunkBuilder, adjacent: &AdjacentBlocks, Vec3 { x, y, z }: Vec3) {
        // Left
        if !adjacent.left {
            let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }

        // Right
        if !adjacent.right {
            let a = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Top
        if !adjacent.top {
            let a = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [0.0, 1.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [0.0, 1.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [0.0, 1.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [0.0, 1.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }

        // Bottom
        if !adjacent.bottom {
            let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [0.0, -1.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [0.0, -1.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [0.0, -1.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [0.0, -1.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Front
        if !adjacent.front {
            let a = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Back
        if !adjacent.back {
            let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [0.0, 0.0, -1.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [0.0, 0.0, -1.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [0.0, 0.0, -1.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [0.0, 0.0, -1.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }
    }
}
