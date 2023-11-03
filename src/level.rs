use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use noise::Perlin;
use parking_lot::{Mutex, RwLock};
use rusqlite::Connection;

use crate::{block_registry::BlockRegistry, config::Config, player::Player, position::ChunkPos};

mod adjacent_sides;
mod chunk;
mod chunk_data;
mod chunk_generator;
mod chunk_loader;
mod mesh_builder;
mod visible_chunks;

pub use adjacent_sides::AdjacentBlocks;
pub use chunk_data::CHUNK_SIZE;
pub use mesh_builder::*;

use self::{chunk::Chunk, chunk_loader::ChunkLoader, visible_chunks::visible_chunks};

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct Level(Arc<RwLock<LevelInner>>);

impl Level {
    pub fn new(inner: LevelInner) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }
}

pub struct LevelInner {
    db: Mutex<Connection>,
    loaded_chunks: HashMap<ChunkPos, Chunk>,
    perlin_noise: Perlin,
}

impl LevelInner {
    fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.loaded_chunks.get(&pos)
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_level)
            .add_systems(Update, load_chunks);
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
        db: Mutex::new(connection),
        loaded_chunks: HashMap::new(),
        perlin_noise: Perlin::default(),
    });

    let chunk_loader = ChunkLoader::new(level.clone(), registry.clone());
    commands.insert_resource(chunk_loader);
}

fn load_chunks(
    mut commands: Commands,
    chunk_loader: Res<ChunkLoader>,
    config: Res<Config>,
    level: Res<Level>,
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
            .insert(TransformBundle::from_transform(transform))
            .insert(VisibilityBundle::default());

        chunk_loader.load(pos);
    }

    for (entity, pos) in chunks.iter().filter(|ex| !visible.contains(ex.1)) {
        level.write().loaded_chunks.remove(pos);
        commands.entity(entity).despawn_recursive();
    }
}
