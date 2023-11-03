use bevy::prelude::{Mesh, Vec3};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};
use itertools::Itertools;

use crate::{block_registry::BlockRegistry, position::ChunkPos};

use super::{chunk::Chunk, AdjacentBlocks, Level, MeshBuilder, CHUNK_SIZE};

pub fn mesh_chunk(
    level: Level,
    pos: ChunkPos,
    chunk: Chunk,
    registry: BlockRegistry,
) -> (Mesh, Option<Collider>) {
    let mut mesh_builder = MeshBuilder::new();

    let level = level.read();
    let left = level.chunk(pos.left()).cloned();
    let right = level.chunk(pos.right()).cloned();
    let top = level.chunk(pos.top()).cloned();
    let bottom = level.chunk(pos.bottom()).cloned();
    let front = level.chunk(pos.front()).cloned();
    let back = level.chunk(pos.back()).cloned();
    drop(level);

    for ((x, y), z) in (0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
    {
        let Some(block) = chunk.read().block(x, y, z) else {
            continue;
        };

        let adjacent_sides = AdjacentBlocks {
            left: if x == 0 {
                left.as_ref()
                    .map(|chunk| chunk.read().block(CHUNK_SIZE - 1, y, z).is_some())
                    .unwrap_or(false)
            } else {
                chunk.read().block(x - 1, y, z).is_some()
            },
            right: if x == CHUNK_SIZE - 1 {
                right
                    .as_ref()
                    .map(|chunk| chunk.read().block(0, y, z).is_some())
                    .unwrap_or(false)
            } else {
                chunk.read().block(x + 1, y, z).is_some()
            },
            bottom: if y == 0 {
                bottom
                    .as_ref()
                    .map(|chunk| chunk.read().block(x, CHUNK_SIZE - 1, z).is_some())
                    .unwrap_or(false)
            } else {
                chunk.read().block(x, y - 1, z).is_some()
            },
            top: if y == CHUNK_SIZE - 1 {
                top.as_ref()
                    .map(|chunk| chunk.read().block(x, 0, z).is_some())
                    .unwrap_or(false)
            } else {
                chunk.read().block(x, y + 1, z).is_some()
            },
            back: if z == 0 {
                back.as_ref()
                    .map(|chunk| chunk.read().block(x, y, CHUNK_SIZE - 1).is_some())
                    .unwrap_or(false)
            } else {
                chunk.read().block(x, y, z - 1).is_some()
            },
            front: if z == CHUNK_SIZE - 1 {
                front
                    .as_ref()
                    .map(|chunk| chunk.read().block(x, y, 0).is_some())
                    .unwrap_or(false)
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
