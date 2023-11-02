use bevy::prelude::Resource;
use noise::Perlin;
use sqlx::{SqliteConnection, SqlitePool};
use tokio::sync::mpsc::{self, Sender};

use crate::{block_registry::BlockRegistry, position::ChunkPos};

use super::{chunk::Chunk, chunk_data::ChunkData, chunk_generator::generate_chunk, Level};

#[derive(Resource, Clone)]
pub struct ChunkLoader(Sender<ChunkPos>);

impl ChunkLoader {
    pub fn new(level: Level, pool: SqlitePool, registry: BlockRegistry) -> Self {
        let (sender, mut receiver) = mpsc::channel(100);
        tokio::spawn(async move {
            while let Some(pos) = receiver.recv().await {
                if level.read().loaded_chunks.contains_key(&pos) {
                    continue;
                }

                let noise = level.read().perlin_noise;
                let chunk_data = load_chunk(&pool, noise, &registry, pos).await;

                level
                    .write()
                    .loaded_chunks
                    .insert(pos, Chunk::new(chunk_data));
            }
        });
        Self(sender)
    }

    pub async fn load(&self, pos: ChunkPos) {
        self.0.send(pos).await.unwrap();
    }
}

async fn load_chunk(
    pool: &SqlitePool,
    noise: Perlin,
    registry: &BlockRegistry,
    pos: ChunkPos,
) -> ChunkData {
    let mut conn = pool.acquire().await.unwrap();

    if let Some(bytes) = load_chunk_data(&mut conn, pos).await {
        ChunkData::deserialize(&bytes, &registry)
    } else {
        let chunk = generate_chunk(&noise, &registry, pos);
        save_chunk_data(&mut conn, pos, chunk.serialize(&registry)).await;
        chunk
    }
}

async fn save_chunk_data(conn: &mut SqliteConnection, pos: ChunkPos, data: Vec<u8>) {
    sqlx::query!(
        r#"
            REPLACE INTO chunks (x, y, z, data)
            VALUES (?1, ?2, ?3, ?4)
        "#,
        pos.x,
        pos.y,
        pos.z,
        data
    )
    .execute(conn)
    .await;
}

async fn load_chunk_data(conn: &mut SqliteConnection, pos: ChunkPos) -> Option<Vec<u8>> {
    sqlx::query!(
        r#"
            SELECT data FROM chunks WHERE
            x = ?1 AND
            y = ?2 AND
            z = ?3
        "#,
        pos.x,
        pos.y,
        pos.z
    )
    .fetch_one(conn)
    .await
    .map(|chunk| chunk.data)
    .ok()
    .flatten()
}
