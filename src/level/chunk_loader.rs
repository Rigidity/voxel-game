use noise::Perlin;
use sqlx::{SqliteConnection, SqlitePool};

use crate::{block_registry::SharedBlockRegistry, position::ChunkPos};

use super::{chunk_data::ChunkData, chunk_generator::generate_chunk};

pub async fn load_chunk(
    pool: SqlitePool,
    noise: Perlin,
    registry: SharedBlockRegistry,
    pos: ChunkPos,
) -> ChunkData {
    let mut conn = pool.acquire().await.unwrap();

    if let Some(bytes) = load_chunk_data(&mut conn, pos).await {
        ChunkData::deserialize(&bytes, &registry.read())
    } else {
        let chunk = generate_chunk(&noise, &registry, pos);
        save_chunk_data(&mut conn, pos, chunk.serialize(&registry.read())).await;
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
