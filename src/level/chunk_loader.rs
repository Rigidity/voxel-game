use std::sync::mpsc::{self, Sender};

use bevy::prelude::Resource;
use noise::Perlin;

use crate::{block_registry::BlockRegistry, position::ChunkPos};

use super::{
    chunk::Chunk, chunk_data::ChunkData, chunk_generator::generate_chunk, Database, Level,
};

#[derive(Resource, Clone)]
pub struct ChunkLoader(Sender<ChunkPos>);

impl ChunkLoader {
    pub fn new(level: Level, db: Database, registry: BlockRegistry) -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(pos) = receiver.recv() {
                if let Some(chunk) = level.read().loaded_chunks.get(&pos).cloned() {
                    save_chunk_data(&db, pos, chunk.read().serialize(&registry));
                    continue;
                }

                let noise = level.read().perlin_noise;
                let chunk_data = load_chunk(&db, noise, &registry, pos);

                level
                    .write()
                    .loaded_chunks
                    .insert(pos, Chunk::new(chunk_data));
            }
        });
        Self(sender)
    }

    pub fn queue(&self, pos: ChunkPos) {
        self.0.send(pos).unwrap();
    }
}

fn load_chunk(db: &Database, noise: Perlin, registry: &BlockRegistry, pos: ChunkPos) -> ChunkData {
    if let Some(bytes) = load_chunk_data(db, pos) {
        ChunkData::deserialize(&bytes, registry)
    } else {
        let chunk = generate_chunk(&noise, registry, pos);
        save_chunk_data(db, pos, chunk.serialize(registry));
        chunk
    }
}

fn save_chunk_data(db: &Database, pos: ChunkPos, data: Vec<u8>) {
    db.lock()
        .execute(
            "REPLACE INTO chunks (x, y, z, data) VALUES (?1, ?2, ?3, ?4)",
            (pos.x, pos.y, pos.z, data),
        )
        .unwrap();
}

fn load_chunk_data(db: &Database, pos: ChunkPos) -> Option<Vec<u8>> {
    db.lock()
        .query_row(
            "SELECT data FROM chunks WHERE x = ?1 AND y = ?2 AND z = ?3",
            (pos.x, pos.y, pos.z),
            |row| row.get(0),
        )
        .ok()
        .flatten()
}
