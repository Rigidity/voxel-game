use crate::chunk_builder::ChunkBuilder;

pub trait Block {
    fn render(chunk: &mut ChunkBuilder, position: [f32; 3]);
}

pub struct BasicBlock;

impl Block for BasicBlock {
    fn render(chunk: &mut ChunkBuilder, [x, y, z]: [f32; 3]) {
        // Left
        let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);

        // Right
        let a = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, b, c, c, d, a]);

        // Top
        let a = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [0.0, 1.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [0.0, 1.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [0.0, 1.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [0.0, 1.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);

        // Bottom
        let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [0.0, -1.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [0.0, -1.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [0.0, -1.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [0.0, -1.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, b, c, c, d, a]);

        // Front
        let a = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]);
        let d = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]);
        chunk.indices([a, b, c, c, d, a]);

        // Back
        let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [0.0, 0.0, -1.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [0.0, 0.0, -1.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [0.0, 0.0, -1.0], [1.0, 1.0]);
        let d = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [0.0, 0.0, -1.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);
    }
}
