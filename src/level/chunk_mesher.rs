use bevy::prelude::{Mesh, Vec3};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};
use itertools::Itertools;

use crate::block_registry::BlockRegistry;

use super::{
    adjacent_sides::AdjacentChunkData, chunk::Chunk, AdjacentBlocks, MeshBuilder, CHUNK_SIZE,
};

pub fn mesh_chunk(
    adjacent: AdjacentChunkData,
    chunk: Chunk,
    registry: BlockRegistry,
) -> (Mesh, Option<Collider>) {
    let mut mesh_builder = MeshBuilder::new();

    for ((x, y), z) in (0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
    {
        let Some(block) = chunk.read().block(x, y, z) else {
            continue;
        };

        let adjacent_sides = AdjacentBlocks {
            left: if x == 0 {
                adjacent.left.map(|data| data[y][z]).unwrap_or(false)
            } else {
                chunk.read().block(x - 1, y, z).is_some()
            },
            right: if x == CHUNK_SIZE - 1 {
                adjacent.right.map(|data| data[y][z]).unwrap_or(false)
            } else {
                chunk.read().block(x + 1, y, z).is_some()
            },
            bottom: if y == 0 {
                adjacent.bottom.map(|data| data[x][z]).unwrap_or(false)
            } else {
                chunk.read().block(x, y - 1, z).is_some()
            },
            top: if y == CHUNK_SIZE - 1 {
                adjacent.top.map(|data| data[x][z]).unwrap_or(false)
            } else {
                chunk.read().block(x, y + 1, z).is_some()
            },
            back: if z == 0 {
                adjacent.back.map(|data| data[x][y]).unwrap_or(false)
            } else {
                chunk.read().block(x, y, z - 1).is_some()
            },
            front: if z == CHUNK_SIZE - 1 {
                adjacent.front.map(|data| data[x][y]).unwrap_or(false)
            } else {
                chunk.read().block(x, y, z + 1).is_some()
            },
        };

        let translation = Vec3::new(x as f32, y as f32, z as f32);
        (registry.read().block(block).render)(&mut mesh_builder, adjacent_sides, translation);
    }

    let mesh = mesh_builder.build();
    let mut collider = None;

    if mesh.count_vertices() > 0 {
        collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh);
    }

    (mesh, collider)
}
