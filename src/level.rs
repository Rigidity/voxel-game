use std::sync::Arc;

use async_io::block_on;
use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    render::primitives::Aabb,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_rapier3d::prelude::{Collider, Friction};
use futures_lite::future;
use itertools::Itertools;
use noise::Perlin;
use parking_lot::{Mutex, RwLock};
use rusqlite::Connection;

use crate::{
    block_registry::BlockRegistry,
    config::Config,
    level::{
        chunk_data::ChunkData, chunk_generator::generate_chunk, chunk_loader::load_chunk_data,
        chunk_mesher::mesh_chunk,
    },
    player::Player,
    position::ChunkPos,
    ChunkMaterial,
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
pub use chunk::*;
pub use chunk_data::CHUNK_SIZE;
pub use chunk_loader::save_chunk_data;
pub use mesh_builder::*;

use self::{chunk::Chunk, visible_chunks::visible_chunks};

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

impl LevelInner {
    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.loaded_chunks.get(&pos)
    }
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

fn create_level(mut commands: Commands) {
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

    commands.insert_resource(level);
    commands.insert_resource(db);
}

#[derive(Component)]
struct LoadTask(Task<ChunkData>);

fn load_chunks(
    mut commands: Commands,
    config: Res<Config>,
    level: Res<Level>,
    db: Res<Database>,
    registry: Res<BlockRegistry>,
    chunk_material: Res<ChunkMaterial>,
    player: Query<&Transform, With<Player>>,
    mut chunks: Query<(Entity, &ChunkPos, Option<&mut LoadTask>)>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let transform = player.single();
    let chunk_pos = ChunkPos::from(transform.translation);

    let visible = visible_chunks(chunk_pos, config.render_distance);

    for &pos in visible
        .iter()
        .filter(|pos| !chunks.iter().any(|ex| ex.1 == *pos))
    {
        let transform = Transform::from_translation(pos.into());
        let perlin_noise = level.read().perlin_noise;
        let registry = registry.clone();
        let db = db.clone();

        let task = task_pool.spawn(async move {
            match load_chunk_data(&db, pos) {
                Some(bytes) => ChunkData::deserialize(&bytes, &registry),
                None => {
                    let chunk_data = generate_chunk(&perlin_noise, &registry, pos);
                    save_chunk_data(&db, pos, chunk_data.serialize(&registry));
                    chunk_data
                }
            }
        });

        commands
            .spawn(pos)
            .insert(chunk_material.handle.clone())
            .insert(TransformBundle::from_transform(transform))
            .insert(VisibilityBundle::default())
            .insert(Friction::new(0.0))
            .insert(LoadTask(task));
    }

    for (entity, pos, load_task) in chunks.iter_mut() {
        if visible.contains(pos) {
            if let Some(chunk_data) =
                load_task.and_then(|mut task| block_on(future::poll_once(&mut task.0)))
            {
                let mut entity = commands.entity(entity);
                entity.remove::<LoadTask>();

                level
                    .write()
                    .loaded_chunks
                    .insert(*pos, Chunk::new(chunk_data));
            }
        } else {
            level.write().loaded_chunks.remove(pos);
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct MeshTask(Task<(Mesh, Option<Collider>)>);

fn mesh_chunks(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    level: Res<Level>,
    chunks: Query<(Entity, &ChunkPos)>,
    need_mesh: Query<
        &ChunkPos,
        Or<(
            (Without<Handle<Mesh>>, Without<MeshTask>, Without<LoadTask>),
            With<Dirty>,
        )>,
    >,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, &pos) in need_mesh
        .iter()
        .flat_map(|pos| pos.adjacent_chunks().into_iter().chain([*pos]))
        .unique()
        .filter_map(|pos| chunks.iter().find(|ex| *ex.1 == pos))
    {
        let Some(chunk) = level.read().loaded_chunks.get(&pos).cloned() else {
            continue;
        };

        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        spawn_chunk_mesh_task(
            &mut entity,
            task_pool,
            level.clone(),
            registry.clone(),
            pos,
            chunk,
        );
    }
}

fn spawn_chunk_mesh_task(
    entity: &mut EntityCommands,
    task_pool: &AsyncComputeTaskPool,
    level: Level,
    registry: BlockRegistry,
    pos: ChunkPos,
    chunk: Chunk,
) {
    let task = task_pool.spawn(async move { mesh_chunk(level, pos, chunk, registry) });

    entity.insert(MeshTask(task)).remove::<Dirty>();
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
