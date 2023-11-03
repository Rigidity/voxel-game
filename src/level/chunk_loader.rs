use std::sync::mpsc::{self, Sender};

use bevy::prelude::Resource;
use noise::Perlin;

use crate::{block_registry::BlockRegistry, position::ChunkPos};

use super::{chunk::Chunk, chunk_data::ChunkData, chunk_generator::generate_chunk, Level};

#[derive(Resource, Clone)]
pub struct ChunkLoader(Sender<ChunkPos>);

impl ChunkLoader {
    pub fn new(level: Level, registry: BlockRegistry) -> Self {
        let (sender, mut receiver) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(pos) = receiver.recv() {
                if level.read().loaded_chunks.contains_key(&pos) {
                    continue;
                }

                let noise = level.read().perlin_noise;
                let chunk_data = load_chunk(&level, noise, &registry, pos);

                level
                    .write()
                    .loaded_chunks
                    .insert(pos, Chunk::new(chunk_data));
            }
        });
        Self(sender)
    }

    pub fn load(&self, pos: ChunkPos) {
        self.0.send(pos).unwrap();
    }
}

fn load_chunk(level: &Level, noise: Perlin, registry: &BlockRegistry, pos: ChunkPos) -> ChunkData {
    if let Some(bytes) = load_chunk_data(level, pos) {
        ChunkData::deserialize(&bytes, registry)
    } else {
        let chunk = generate_chunk(&noise, registry, pos);
        save_chunk_data(level, pos, chunk.serialize(registry));
        chunk
    }
}

fn save_chunk_data(level: &Level, pos: ChunkPos, data: Vec<u8>) {
    level
        .read()
        .db
        .lock()
        .execute(
            "REPLACE INTO chunks (x, y, z, data) VALUES (?1, ?2, ?3, ?4)",
            (pos.x, pos.y, pos.z, data),
        )
        .unwrap();
}

fn load_chunk_data(level: &Level, pos: ChunkPos) -> Option<Vec<u8>> {
    level
        .read()
        .db
        .lock()
        .query_row(
            "SELECT data FROM chunks WHERE x = ?1 AND y = ?2 AND z = ?3",
            (pos.x, pos.y, pos.z),
            |row| row.get(0),
        )
        .ok()
        .flatten()
}
