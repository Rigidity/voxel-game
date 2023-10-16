use crate::chunk_builder::ChunkBuilder;

pub trait Block {
    fn render(chunk: &mut ChunkBuilder, block_pos: BlockPos);
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos(u8, u8, u8);

impl BlockPos {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        Self(x, y, z)
    }

    pub fn x(&self) -> u8 {
        self.0
    }

    pub fn y(&self) -> u8 {
        self.1
    }

    pub fn z(&self) -> u8 {
        self.2
    }
}

pub struct BasicBlock;

impl Block for BasicBlock {
    fn render(chunk: &mut ChunkBuilder, block_pos: BlockPos) {
        let x = block_pos.x() as f32;
        let y = block_pos.y() as f32;
        let z = block_pos.z() as f32;

        // Left
        let a = chunk.vertex([x - 0.5, y - 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x - 0.5, y + 0.5, z - 0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x - 0.5, y + 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x - 0.5, y - 0.5, z + 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);

        // Right
        let a = chunk.vertex([x + 0.5, y - 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 0.5, y + 0.5, z - 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 0.5, y + 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x + 0.5, y - 0.5, z + 0.5], [1.0, 0.0, 0.0], [1.0, 0.0]);
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
