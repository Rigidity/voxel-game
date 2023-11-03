use std::sync::Arc;

use async_io::block_on;
use bevy::{
    prelude::*,
    render::primitives::Aabb,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_rapier3d::prelude::{Collider, Friction};
use futures_lite::future;
use noise::Perlin;
use parking_lot::{Mutex, RwLock};
use rusqlite::Connection;

use crate::{
    block_registry::BlockRegistry, config::Config, level::chunk_mesher::mesh_chunk, player::Player,
    position::ChunkPos, ChunkMaterial,
};

mod adjacent_sides;
mod chunk;
mod chunk_data;
mod chunk_generator;
mod chunk_loader;
mod chunk_mesher;
mod mesh_builder;
mod visible_chunks;

pub use adjacent_sides::AdjacentBlocks;
pub use chunk_data::CHUNK_SIZE;
pub use mesh_builder::*;

use self::{
    adjacent_sides::AdjacentChunkData, chunk::Chunk, chunk_loader::ChunkLoader,
    visible_chunks::visible_chunks,
};

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct Level(Arc<RwLock<LevelInner>>);

impl Level {
    pub fn new(inner: LevelInner) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }
}

pub struct LevelInner {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
    perlin_noise: Perlin,
}

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct Database(Arc<Mutex<Connection>>);

impl Database {
    pub fn new(connection: Connection) -> Self {
        Self(Arc::new(Mutex::new(connection)))
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_level).add_systems(
            Update,
            (load_chunks, apply_deferred, (mesh_chunks, insert_meshes)).chain(),
        );
    }
}

fn create_level(mut commands: Commands, registry: Res<BlockRegistry>) {
    let connection = Connection::open("chunks.sqlite").unwrap();

    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                x INTEGER,
                y INTEGER,
                z INTEGER,
                data BLOB,
                UNIQUE(x, y, z)
            )",
            (),
        )
        .unwrap();

    let level = Level::new(LevelInner {
        loaded_chunks: HashMap::new(),
        perlin_noise: Perlin::default(),
    });
    let db = Database::new(connection);
    let chunk_loader = ChunkLoader::new(level.clone(), db.clone(), registry.clone());

    commands.insert_resource(level);
    commands.insert_resource(db);
    commands.insert_resource(chunk_loader);
}

fn load_chunks(
    mut commands: Commands,
    chunk_loader: Res<ChunkLoader>,
    config: Res<Config>,
    level: Res<Level>,
    chunk_material: Res<ChunkMaterial>,
    chunks: Query<(Entity, &ChunkPos)>,
    player: Query<&Transform, With<Player>>,
) {
    let transform = player.single();
    let chunk_pos = ChunkPos::from(transform.translation);

    let visible = visible_chunks(chunk_pos, config.render_distance);

    for &pos in visible
        .iter()
        .filter(|pos| !chunks.iter().any(|ex| ex.1 == *pos))
    {
        let transform = Transform::from_translation(pos.into());

        commands
            .spawn(pos)
            .insert(chunk_material.handle.clone())
            .insert(TransformBundle::from_transform(transform))
            .insert(VisibilityBundle::default())
            .insert(Friction::new(0.0));

        chunk_loader.load(pos);
    }

    for (entity, pos) in chunks.iter().filter(|ex| !visible.contains(ex.1)) {
        level.write().loaded_chunks.remove(pos);
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct MeshTask(Task<(Mesh, Option<Collider>)>);

type NeedsMesh = (Without<Handle<Mesh>>, Without<MeshTask>);

fn mesh_chunks(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    level: Res<Level>,
    chunks: Query<(Entity, &ChunkPos), NeedsMesh>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, &pos) in chunks.iter() {
        let Some(chunk) = level.read().loaded_chunks.get(&pos).cloned() else {
            continue;
        };

        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        macro_rules! adjacent_faces {
            ( $main:ident, $( $name:ident, $pos:expr, |$row_name:ident, $cell_name:ident|
                    => [$x:expr, $y:expr, $z:expr]; )* ) => {
                $( let $name = level.read().loaded_chunks.get(&$pos).map(|chunk| {
                    let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                    for ($row_name, data) in data.iter_mut().enumerate() {
                        for ($cell_name, data) in data.iter_mut().enumerate() {
                            *data = chunk.read().block($x, $y, $z).is_some();
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

        let registry = registry.clone();
        let task = thread_pool.spawn(async move { mesh_chunk(adjacent, chunk, registry) });

        entity.insert(MeshTask(task));
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
