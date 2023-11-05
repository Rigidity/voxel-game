use crate::position::ChunkPos;

use super::Database;

pub fn save_chunk_data(db: &Database, pos: ChunkPos, data: Vec<u8>) {
    db.lock()
        .execute(
            "REPLACE INTO chunks (x, y, z, data) VALUES (?1, ?2, ?3, ?4)",
            (pos.x, pos.y, pos.z, data),
        )
        .unwrap();
}

pub fn load_chunk_data(db: &Database, pos: ChunkPos) -> Option<Vec<u8>> {
    db.lock()
        .query_row(
            "SELECT data FROM chunks WHERE x = ?1 AND y = ?2 AND z = ?3",
            (pos.x, pos.y, pos.z),
            |row| row.get(0),
        )
        .ok()
        .flatten()
}
