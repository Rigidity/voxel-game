use std::{
    fs,
    sync::{Arc, RwLock},
};

use async_io::block_on;
use bevy::{
    prelude::*,
    render::primitives::Aabb,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::prelude::*;
use futures_lite::future;
use itertools::Itertools;
use noise::{NoiseFn, Perlin};

use crate::{
    block_registry::{BlockRegistry, SharedBlockRegistry},
    level::{Chunk, Dirty, Level, CHUNK_SIZE},
    player::Player,
    position::{BlockPos, ChunkPos},
};

use super::{build_chunk, AdjacentChunkData};

#[derive(Component)]
pub struct MeshTask(Task<(Mesh, Option<Collider>)>);

#[derive(Component)]
pub struct GenerateTask(Task<Chunk>);

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
                (load_chunks, add_chunks, remove_chunks),
                apply_deferred,
                (generate_meshes, insert_meshes),
            )
                .chain(),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn load_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
    registry: Res<SharedBlockRegistry>,
    max_distance: Res<ChunkDistance>,
    chunks: Query<&ChunkPos>,
    player: Query<&Transform, With<Player>>,
    server: Res<AssetServer>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
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
    let center_pos = player_chunk_pos.center();

    let d = max_distance.0 as i32;

    let positions =
        // x
        (-d..=d).flat_map(|x| {
            // y
            (-d..=d).flat_map(move |y| {
                // z
                (-d..=d).map(move |z| player_chunk_pos + ChunkPos::new(x, y, z))
            })
        })
        .filter(|pos| !chunks.iter().any(|item| item == pos))
        .filter(|pos| center_pos.distance(pos.center()) <= block_distance)
        .collect_vec();

    for pos in positions {
        let material = StandardMaterial {
            base_color_texture: Some(handle.clone()),
            perceptual_roughness: 1.0,
            reflectance: 0.2,
            ..default()
        };

        let mut entity = commands.spawn((
            pos,
            materials.add(material),
            TransformBundle::from_transform(Transform::from_translation(Vec3::from(
                BlockPos::from(pos),
            ))),
            VisibilityBundle::default(),
            Friction::new(0.25),
            Dirty,
        ));

        if !level.is_loaded(pos) {
            let noise = level.noise();
            let registry_clone = Arc::clone(&registry);
            let task = thread_pool.spawn(async move {
                fs::create_dir_all("chunks").unwrap();
                if let Ok(bytes) =
                    fs::read(format!("chunks/chunk_{}_{}_{}.bin", pos.x, pos.y, pos.z))
                {
                    Chunk::deserialize(&bytes, &registry_clone.read().unwrap())
                } else {
                    generate_chunk(noise, pos, registry_clone)
                }
            });
            entity.insert(GenerateTask(task));
        }
    }
}

fn generate_chunk(
    noise: Perlin,
    chunk_pos: ChunkPos,
    registry: Arc<RwLock<BlockRegistry>>,
) -> Chunk {
    let dirt = registry.read().unwrap().block_id("dirt");
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let block_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let block_z = chunk_pos.z * CHUNK_SIZE as i32 + z as i32;
            let noise = noise.get([block_x as f64 / 90.0, block_z as f64 / 90.0]);
            for y in 0..CHUNK_SIZE {
                let block_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;
                if block_y as f64 <= noise * 18.0 {
                    *chunk.block_mut(x, y, z) = Some(dirt);
                }
            }
        }
    }

    fs::create_dir_all("chunks").unwrap();
    fs::write(
        format!(
            "chunks/chunk_{}_{}_{}.bin",
            chunk_pos.x, chunk_pos.y, chunk_pos.z
        ),
        chunk.serialize(&registry.read().unwrap()),
    )
    .unwrap();

    chunk
}

fn add_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut query: Query<(Entity, &ChunkPos, &mut GenerateTask)>,
) {
    for (entity, &chunk_pos, mut generate_task) in query.iter_mut() {
        if let Some(chunk) = block_on(future::poll_once(&mut generate_task.0)) {
            let mut entity = commands.entity(entity);
            entity.remove::<GenerateTask>();
            level.add_chunk(chunk_pos, chunk)
        }
    }
}

fn remove_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    max_distance: Res<ChunkDistance>,
    chunks: Query<(Entity, &ChunkPos)>,
    player: Query<&Transform, With<Player>>,
) {
    let max_distance = (max_distance.0 * CHUNK_SIZE) as f32;
    let transform = player.single();
    let player_chunk_pos = BlockPos::from(transform.translation).chunk_pos().0;
    let player_center_pos = player_chunk_pos.center();

    for (chunk, chunk_pos) in chunks.iter() {
        if player_center_pos.distance(chunk_pos.center()) > max_distance {
            commands.entity(chunk).despawn_recursive();
            level.remove_chunk(chunk_pos);
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
    registry: Res<SharedBlockRegistry>,
    level: Res<Level>,
    query: Query<(Entity, &ChunkPos), With<Dirty>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, &chunk_pos) in query.iter() {
        let Some(chunk) = level.chunk(chunk_pos).cloned() else {
            continue;
        };

        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        let left = level.chunk(chunk_pos - ChunkPos::X).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (y, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(CHUNK_SIZE - 1, y, z).is_some();
                }
            }
            data
        });

        let right = level.chunk(chunk_pos + ChunkPos::X).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (y, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(0, y, z).is_some();
                }
            }
            data
        });

        let top = level.chunk(chunk_pos + ChunkPos::Y).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, 0, z).is_some();
                }
            }
            data
        });

        let bottom = level.chunk(chunk_pos - ChunkPos::Y).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, CHUNK_SIZE - 1, z).is_some();
                }
            }
            data
        });

        let front = level.chunk(chunk_pos + ChunkPos::Z).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (y, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, y, 0).is_some();
                }
            }
            data
        });

        let back = level.chunk(chunk_pos - ChunkPos::Z).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (y, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, y, CHUNK_SIZE - 1).is_some();
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

        let registry_clone = registry.clone();
        let task = thread_pool.spawn(async move { build_chunk(adjacent, chunk, registry_clone) });

        entity.remove::<Dirty>().insert(MeshTask(task));
    }
}
