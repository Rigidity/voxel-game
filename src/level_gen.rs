use std::sync::Arc;

use async_io::block_on;
use bevy::{
    prelude::*,
    render::primitives::Aabb,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::prelude::*;
use futures_lite::future;

use crate::{
    chunk::{Dirty, CHUNK_SIZE},
    chunk_builder::{build_chunk, AdjacentChunkData},
    level::Level,
    player::Player,
    position::{BlockPos, ChunkPos},
};

#[derive(Component)]
pub struct MeshTask(Task<(Mesh, Option<Collider>)>);

#[derive(Resource)]
struct ChunkDistance(usize);

impl Default for ChunkDistance {
    fn default() -> Self {
        Self(8)
    }
}

pub struct LevelGenPlugin;

impl Plugin for LevelGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkDistance>().add_systems(
            Update,
            (
                (load_chunks, unload_chunks),
                apply_deferred,
                (generate_meshes, insert_meshes),
            )
                .chain(),
        );
    }
}

fn load_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level: ResMut<Level>,
    max_distance: Res<ChunkDistance>,
    player: Query<&Transform, With<Player>>,
    server: Res<AssetServer>,
) {
    let handle = server.load("blocks/dirt.png");
    let block_distance = (max_distance.0 * CHUNK_SIZE) as f32;
    let transform = player.single();

    let player_chunk_pos = BlockPos::new(
        transform.translation.x as i32,
        transform.translation.y as i32,
        transform.translation.z as i32,
    )
    .chunk_pos()
    .0;
    let player_center_pos = player_chunk_pos.center();

    let distance = max_distance.0 as i32;

    for x in -distance..=distance {
        for y in -distance..=distance {
            for z in -distance..=distance {
                let chunk_pos = player_chunk_pos.clone() + ChunkPos::new(x, y, z);

                if level.chunk(&chunk_pos).is_some() {
                    continue;
                }

                if player_center_pos.distance(chunk_pos.center()) <= block_distance {
                    level.load_chunk(&chunk_pos);

                    let material = StandardMaterial {
                        base_color_texture: Some(handle.clone()),
                        perceptual_roughness: 1.0,
                        reflectance: 0.2,
                        ..default()
                    };

                    commands.spawn((
                        chunk_pos.clone(),
                        materials.add(material),
                        TransformBundle::from_transform(Transform::from_xyz(
                            (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                        )),
                        VisibilityBundle::default(),
                        Friction::new(0.25),
                        Dirty,
                    ));
                }
            }
        }
    }
}

fn unload_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    max_distance: Res<ChunkDistance>,
    chunks: Query<(Entity, &ChunkPos)>,
    player: Query<&Transform, With<Player>>,
) {
    let max_distance = (max_distance.0 * CHUNK_SIZE) as f32;
    let transform = player.single();
    let player_chunk_pos = BlockPos::new(
        transform.translation.x as i32,
        transform.translation.y as i32,
        transform.translation.z as i32,
    )
    .chunk_pos()
    .0;
    let player_center_pos = player_chunk_pos.center();

    for (chunk, chunk_pos) in chunks.iter() {
        if player_center_pos.distance(chunk_pos.center()) > max_distance {
            commands.entity(chunk).despawn_recursive();
            level.unload_chunk(chunk_pos);
        }
    }
}

fn insert_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut MeshTask)>,
) {
    for (entity, mut mesh_task) in query.iter_mut() {
        if let Some((mesh, collider)) = block_on(future::poll_once(&mut mesh_task.0)) {
            let mut entity = commands.entity(entity);

            entity.remove::<MeshTask>();
            entity.insert(meshes.add(mesh)).remove::<Aabb>();

            if let Some(collider) = collider {
                entity.insert(collider);
            } else {
                entity.remove::<Collider>();
            }
        }
    }
}

fn generate_meshes(
    mut commands: Commands,
    level: Res<Level>,
    query: Query<(Entity, &ChunkPos), With<Dirty>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, chunk_pos) in query.iter() {
        let Some(chunk) = level.chunk(chunk_pos).map(Arc::clone) else {
            continue;
        };

        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        let left = level
            .chunk(&(chunk_pos.clone() - ChunkPos::X))
            .map(Arc::clone)
            .map(|chunk| {
                let lock = chunk.read().unwrap();
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (y, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = lock.block_relative(CHUNK_SIZE - 1, y, z).is_some();
                    }
                }
                data
            });

        let right = level
            .chunk(&(chunk_pos.clone() + ChunkPos::X))
            .map(Arc::clone)
            .map(|chunk| {
                let lock = chunk.read().unwrap();
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (y, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = lock.block_relative(0, y, z).is_some();
                    }
                }
                data
            });

        let top = level
            .chunk(&(chunk_pos.clone() + ChunkPos::Y))
            .map(Arc::clone)
            .map(|chunk| {
                let lock = chunk.read().unwrap();
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = lock.block_relative(x, 0, z).is_some();
                    }
                }
                data
            });

        let bottom = level
            .chunk(&(chunk_pos.clone() - ChunkPos::Y))
            .map(Arc::clone)
            .map(|chunk| {
                let lock = chunk.read().unwrap();
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = lock.block_relative(x, CHUNK_SIZE - 1, z).is_some();
                    }
                }
                data
            });

        let front = level
            .chunk(&(chunk_pos.clone() + ChunkPos::Z))
            .map(Arc::clone)
            .map(|chunk| {
                let lock = chunk.read().unwrap();
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (y, data) in data.iter_mut().enumerate() {
                        *data = lock.block_relative(x, y, 0).is_some();
                    }
                }
                data
            });

        let back = level
            .chunk(&(chunk_pos.clone() - ChunkPos::Z))
            .map(Arc::clone)
            .map(|chunk| {
                let lock = chunk.read().unwrap();
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (y, data) in data.iter_mut().enumerate() {
                        *data = lock.block_relative(x, y, CHUNK_SIZE - 1).is_some();
                    }
                }
                data
            });

        let adjacent = AdjacentChunkData {
            left,
            right,
            top,
            bottom,
            front,
            back,
        };

        let task =
            thread_pool.spawn(async move { build_chunk(adjacent, chunk.clone().read().unwrap()) });

        entity.remove::<Dirty>().insert(MeshTask(task));
    }
}
