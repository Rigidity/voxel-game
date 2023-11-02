use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use noise::Perlin;
use parking_lot::RwLock;

use crate::{
    block_registry::BlockRegistry, config::Config, player::Player, position::ChunkPos, Database,
};

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

#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct Level(Arc<RwLock<LevelInner>>);

#[derive(Default)]
pub struct LevelInner {
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
        app.init_resource::<Level>()
            .add_systems(Startup, spawn_chunk_loader)
            .add_systems(Update, load_chunks);
    }
}

fn spawn_chunk_loader(
    mut commands: Commands,
    level: Res<Level>,
    db: Res<Database>,
    registry: Res<BlockRegistry>,
) {
    let chunk_loader = ChunkLoader::new(level.clone(), db.clone(), registry.clone());
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

    let mut chunks_to_load = Vec::new();

    for &pos in visible
        .iter()
        .filter(|pos| !chunks.iter().any(|ex| ex.1 == *pos))
    {
        let transform = Transform::from_translation(pos.into());

        commands
            .spawn(pos)
            .insert(TransformBundle::from_transform(transform))
            .insert(VisibilityBundle::default());

        chunks_to_load.push(pos);
    }

    let chunk_loader = chunk_loader.clone();
    tokio::spawn(async move {
        for pos in chunks_to_load {
            chunk_loader.load(pos).await;
        }
    });

    for (entity, pos) in chunks.iter().filter(|ex| !visible.contains(ex.1)) {
        level.write().loaded_chunks.remove(pos);
        commands.entity(entity).despawn_recursive();
    }
}
