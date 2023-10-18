use bevy::prelude::Vec3;

use crate::{chunk_builder::ChunkBuilder, level::Level, position::BlockPos};

pub trait Block {
    fn render(level: &Level, chunk: &mut ChunkBuilder, block_pos: &BlockPos, translation: Vec3);
}

pub struct BasicBlock;

impl Block for BasicBlock {
    fn render(
        level: &Level,
        chunk: &mut ChunkBuilder,
        block_pos: &BlockPos,
        Vec3 { x, y, z }: Vec3,
    ) {
        let left = level
            .block(&(block_pos.clone() - BlockPos::X))
            .unwrap_or_default();
        let right = level
            .block(&(block_pos.clone() + BlockPos::X))
            .unwrap_or_default();
        let top = level
            .block(&(block_pos.clone() + BlockPos::Y))
            .unwrap_or_default();
        let bottom = level
            .block(&(block_pos.clone() - BlockPos::Y))
            .unwrap_or_default();
        let front = level
            .block(&(block_pos.clone() + BlockPos::Z))
            .unwrap_or_default();
        let back = level
            .block(&(block_pos.clone() - BlockPos::Z))
            .unwrap_or_default();

        // Left
        if !left {
            let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }

        // Right
        if !right {
            let a = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Top
        if !top {
            let a = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [0.0, 1.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [0.0, 1.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [0.0, 1.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [0.0, 1.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }

        // Bottom
        if !bottom {
            let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [0.0, -1.0, 0.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [0.0, -1.0, 0.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [0.0, -1.0, 0.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [0.0, -1.0, 0.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Front
        if !front {
            let a = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]);
            chunk.indices([a, b, c, c, d, a]);
        }

        // Back
        if !back {
            let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [0.0, 0.0, -1.0], [0.0, 0.0]);
            let b = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [0.0, 0.0, -1.0], [0.0, 1.0]);
            let c = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [0.0, 0.0, -1.0], [1.0, 1.0]);
            let d = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [0.0, 0.0, -1.0], [1.0, 0.0]);
            chunk.indices([a, d, c, c, b, a]);
        }
    }
}
