use bevy::prelude::Vec3;

use crate::level::{AdjacentBlocks, ChunkBuilder};

pub fn render_dirt(chunk: &mut ChunkBuilder, adjacent: AdjacentBlocks, position: Vec3) {
    let x = position.x;
    let y = position.y;
    let z = position.z;

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
