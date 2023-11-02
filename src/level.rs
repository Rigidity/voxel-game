use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use parking_lot::RwLock;

use crate::{config::Config, player::Player, position::ChunkPos};

mod adjacent_sides;
mod chunk;
mod chunk_data;
mod chunk_generator;
mod chunk_loader;
mod mesh_builder;
mod visible_chunks;

pub use chunk_data::CHUNK_SIZE;

use self::{chunk::Chunk, visible_chunks::visible_chunks};

#[derive(Resource, Default, Clone, Deref, DerefMut)]
struct SharedLevel(Arc<RwLock<Level>>);

#[derive(Default)]
struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
}

impl Level {
    fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.loaded_chunks.get(&pos)
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SharedLevel>()
            .add_systems(Update, load_chunks);
    }
}

fn load_chunks(
    mut commands: Commands,
    config: Res<Config>,
    level: Res<SharedLevel>,
    chunks: Query<(Entity, &ChunkPos)>,
    player: Query<&Transform, With<Player>>,
) {
    let transform = player.single();
    let chunk_pos = ChunkPos::from(transform.translation);

    let visible = visible_chunks(chunk_pos, config.render_distance);

    for pos in visible
        .iter()
        .filter(|pos| chunks.iter().any(|ex| ex.1 == *pos))
    {}

    for entity in chunks
        .iter()
        .filter(|ex| !visible.contains(ex.1))
        .map(|ex| ex.0)
    {
        commands.entity(entity).despawn_recursive();
    }
}
