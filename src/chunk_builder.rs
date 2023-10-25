use std::sync::MutexGuard;

use bevy::{
    prelude::*,
    render::{mesh, render_resource::PrimitiveTopology},
};
use bevy_rapier3d::prelude::*;

use crate::chunk::{Chunk, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(u32);

#[derive(Default)]
pub struct ChunkBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct AdjacentBlocks {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
}

impl ChunkBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertex(&mut self, position: [f32; 3], normal: [f32; 3], texcoord: [f32; 2]) -> Index {
        let index = self.positions.len();
        self.positions.push(position);
        self.normals.push(normal);
        self.texcoords.push(texcoord);
        Index(index as u32)
    }

    pub fn indices(&mut self, index: impl IntoIterator<Item = Index>) {
        self.indices.extend(index.into_iter().map(|index| index.0));
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.texcoords);
        mesh.set_indices(Some(mesh::Indices::U32(self.indices)));
        mesh
    }
}

pub struct AdjacentChunkData {
    pub left: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub right: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub top: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub bottom: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub front: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub back: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
}

pub fn build_chunk(
    adjacent: AdjacentChunkData,
    chunk: MutexGuard<Chunk>,
) -> (Mesh, Option<Collider>) {
    let mut chunk_builder = ChunkBuilder::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let Some(block) = chunk.block_relative(x, y, z) else {
                    continue;
                };

                let adjacent_sides = AdjacentBlocks {
                    left: if x == 0 {
                        adjacent.left.map(|data| data[y][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x - 1, y, z).is_some()
                    },
                    right: if x == CHUNK_SIZE - 1 {
                        adjacent.right.map(|data| data[y][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x + 1, y, z).is_some()
                    },
                    bottom: if y == 0 {
                        adjacent.bottom.map(|data| data[x][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y - 1, z).is_some()
                    },
                    top: if y == CHUNK_SIZE - 1 {
                        adjacent.top.map(|data| data[x][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y + 1, z).is_some()
                    },
                    back: if z == 0 {
                        adjacent.back.map(|data| data[x][y]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y, z - 1).is_some()
                    },
                    front: if z == CHUNK_SIZE - 1 {
                        adjacent.front.map(|data| data[x][y]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y, z + 1).is_some()
                    },
                };

                let translation = Vec3::new(x as f32, y as f32, z as f32);

                block.render(&mut chunk_builder, adjacent_sides, translation);
            }
        }
    }

    let mesh = chunk_builder.build();
    let mut collider = None;

    if mesh.count_vertices() > 0 {
        collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh);
    }

    (mesh, collider)
}
