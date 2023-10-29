use std::sync::{Arc, RwLock};

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

    let mut positions =
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

    positions.sort_by(|a, b| {
        a.center()
            .distance(transform.translation)
            .total_cmp(&b.center().distance(transform.translation))
    });

    for pos in positions {
        let material = StandardMaterial {
            base_color_texture: Some(handle.clone()),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
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
            let registry = Arc::clone(&registry);
            let connection = Arc::clone(&level.connection);
            let task = thread_pool.spawn(async move {
                let conn = connection.lock().unwrap();

                let result = conn.query_row(
                    "SELECT `data` FROM `chunks` WHERE `x` = ?1 AND `y` = ?2 AND `z` = ?3",
                    (pos.x, pos.y, pos.z),
                    |row| row.get::<_, Vec<u8>>(0),
                );

                if let Ok(bytes) = result {
                    Chunk::deserialize(&bytes, &registry.read().unwrap())
                } else {
                    generate_chunk(noise, pos, registry)
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

    for (entity, &pos) in query.iter() {
        let Some(chunk) = level.chunk(pos).cloned() else {
            continue;
        };

        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        let left = level.chunk(pos - ChunkPos::X).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (y, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(CHUNK_SIZE - 1, y, z).is_some();
                }
            }
            data
        });

        let right = level.chunk(pos + ChunkPos::X).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (y, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(0, y, z).is_some();
                }
            }
            data
        });

        let top = level.chunk(pos + ChunkPos::Y).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, 0, z).is_some();
                }
            }
            data
        });

        let bottom = level.chunk(pos - ChunkPos::Y).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (z, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, CHUNK_SIZE - 1, z).is_some();
                }
            }
            data
        });

        let front = level.chunk(pos + ChunkPos::Z).map(|chunk| {
            let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (x, data) in data.iter_mut().enumerate() {
                for (y, data) in data.iter_mut().enumerate() {
                    *data = chunk.block(x, y, 0).is_some();
                }
            }
            data
        });

        let back = level.chunk(pos - ChunkPos::Z).map(|chunk| {
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

        let registry = Arc::clone(&registry);
        let connection = Arc::clone(&level.connection);

        let task = thread_pool.spawn(async move {
            let data = chunk.serialize(&registry.read().unwrap());
            let conn = connection.lock().unwrap();

            if conn
                .query_row(
                    "SELECT COUNT(*) FROM `chunks` WHERE `x` = ?1 AND `y` = ?2 AND `z` = ?3",
                    (pos.x, pos.y, pos.z),
                    |row| row.get::<_, usize>(0),
                )
                .unwrap()
                == 0
            {
                conn.execute(
                    "INSERT INTO `chunks` (`x`, `y`, `z`, `data`) VALUES (?1, ?2, ?3, ?4)",
                    (pos.x, pos.y, pos.z, data),
                )
                .unwrap();
            } else {
                conn.execute(
                    "UPDATE `chunks` SET `data` = ?1 WHERE `x` = ?2 AND `y` = ?3 AND `z` = ?4",
                    (data, pos.x, pos.y, pos.z),
                )
                .unwrap();
            }

            drop(conn);
            build_chunk(adjacent, chunk, registry)
        });

        entity.remove::<Dirty>().insert(MeshTask(task));
    }
}
