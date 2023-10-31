use std::sync::{Arc, Mutex, RwLock};

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
use rusqlite::Connection;

use crate::{
    block_registry::{BlockRegistry, SharedBlockRegistry},
    config::Config,
    level::{Chunk, Dirty, Level, CHUNK_SIZE},
    player::Player,
    position::{BlockPos, ChunkPos},
    ChunkMaterial,
};

use super::{build_chunk, AdjacentChunkData};

#[derive(Component)]
pub struct MeshTask(Task<(Mesh, Option<Collider>)>);

#[derive(Component)]
pub struct GenerateTask(Task<Chunk>);

pub struct LevelGenPlugin;

impl Plugin for LevelGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (load_chunks, remove_chunks),
                apply_deferred,
                (add_chunks, generate_meshes, insert_meshes),
            )
                .chain(),
        );
    }
}

fn visible_chunk_positions(player_pos: Vec3, distance: i32) -> Vec<ChunkPos> {
    let chunk_pos = BlockPos::from(player_pos).chunk_pos().0;
    let center_pos = chunk_pos.center();
    let block_distance = distance * CHUNK_SIZE as i32;

    (-distance..=distance)
        .cartesian_product(-distance..=distance)
        .cartesian_product(-distance..=distance)
        .map(|((x, y), z)| chunk_pos + ChunkPos::new(x, y, z))
        .filter(|pos| center_pos.distance(pos.center()) <= block_distance as f32)
        .sorted_by(|a, b| {
            a.center()
                .distance(player_pos)
                .total_cmp(&b.center().distance(player_pos))
        })
        .collect_vec()
}

fn load_chunks(
    mut commands: Commands,
    config: Res<Config>,
    level: Res<Level>,
    chunk_material: Res<ChunkMaterial>,
    registry: Res<SharedBlockRegistry>,
    chunks: Query<&ChunkPos>,
    player: Query<&Transform, With<Player>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let player_pos = player.single().translation;

    for pos in visible_chunk_positions(player_pos, config.render_distance)
        .into_iter()
        .filter(|pos| !chunks.iter().any(|existing| existing == pos))
    {
        let mut entity = commands.spawn(pos);

        entity
            .insert(chunk_material.handle.clone())
            .insert(TransformBundle::from(Transform {
                translation: BlockPos::from(pos).into(),
                ..default()
            }))
            .insert(VisibilityBundle::default())
            .insert(Friction::new(0.25))
            .insert(Dirty);

        let registry = Arc::clone(&registry);
        let connection = Arc::clone(&level.connection);
        let task = thread_pool.spawn(load_chunk(pos, level.noise(), registry, connection));

        entity.insert(GenerateTask(task));
    }
}

async fn load_chunk(
    pos: ChunkPos,
    noise: Perlin,
    registry: Arc<RwLock<BlockRegistry>>,
    connection: Arc<Mutex<Connection>>,
) -> Chunk {
    let result = connection.lock().unwrap().query_row(
        "SELECT `data` FROM `chunks` WHERE `x` = ?1 AND `y` = ?2 AND `z` = ?3",
        (pos.x, pos.y, pos.z),
        |row| row.get::<_, Vec<u8>>(0),
    );

    if let Ok(bytes) = result {
        Chunk::deserialize(&bytes, &registry.read().unwrap())
    } else {
        generate_chunk(noise, pos, registry)
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
    mut loading_chunks: Query<(Entity, &ChunkPos, &mut GenerateTask)>,
    chunks: Query<(Entity, &ChunkPos), (Without<GenerateTask>, Without<Dirty>)>,
) {
    for (entity, &pos, mut generate_task) in loading_chunks.iter_mut() {
        let Some(chunk) = block_on(future::poll_once(&mut generate_task.0)) else {
            continue;
        };

        let mut entity = commands.entity(entity);
        entity.remove::<GenerateTask>();
        level.add_chunk(pos, chunk);

        for (adjacent, &adjacent_pos) in chunks.iter() {
            if !pos.is_adjacent(adjacent_pos) {
                continue;
            }

            commands.entity(adjacent).insert(Dirty);
        }
    }
}

fn remove_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    config: Res<Config>,
    chunks: Query<(Entity, &ChunkPos)>,
    player: Query<&Transform, With<Player>>,
) {
    let max_distance = config.render_distance * CHUNK_SIZE as i32;
    let transform = player.single();
    let player_chunk_pos = BlockPos::from(transform.translation).chunk_pos().0;
    let player_center_pos = player_chunk_pos.center();

    for (chunk, chunk_pos) in chunks.iter() {
        if player_center_pos.distance(chunk_pos.center()) > max_distance as f32 {
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

        macro_rules! adjacent_faces {
            ( $main:ident, $( $name:ident, $pos:expr, |$row_name:ident, $cell_name:ident|
                    => [$x:expr, $y:expr, $z:expr]; )* ) => {
                $( let $name = level.chunk($pos).map(|chunk| {
                    let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                    for ($row_name, data) in data.iter_mut().enumerate() {
                        for ($cell_name, data) in data.iter_mut().enumerate() {
                            *data = chunk.block($x, $y, $z).is_some();
                        }
                    }
                    data
                }); )*

                let $main = AdjacentChunkData { $( $name, )* };
            };
        }

        adjacent_faces!(adjacent,
            left, pos - ChunkPos::X, |y, z| => [CHUNK_SIZE - 1, y, z];
            right, pos + ChunkPos::X, |y, z| => [0, y, z];
            top, pos + ChunkPos::Y, |x, z| => [x, 0, z];
            bottom, pos - ChunkPos::Y, |x, z| => [x, CHUNK_SIZE - 1, z];
            front, pos + ChunkPos::Z, |x, y| => [x, y, 0];
            back, pos - ChunkPos::Z, |x, y| => [x, y, CHUNK_SIZE - 1];
        );

        let registry = Arc::clone(&registry);
        let connection = Arc::clone(&level.connection);
        let task = thread_pool.spawn(save_chunk(pos, chunk, adjacent, registry, connection));

        entity.remove::<Dirty>().insert(MeshTask(task));
    }
}

async fn save_chunk(
    pos: ChunkPos,
    chunk: Chunk,
    adjacent: AdjacentChunkData,
    registry: Arc<RwLock<BlockRegistry>>,
    connection: Arc<Mutex<Connection>>,
) -> (Mesh, Option<Collider>) {
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
}
