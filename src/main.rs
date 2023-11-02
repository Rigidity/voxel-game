use bevy::prelude::*;
use sqlx::{SqliteConnection, SqlitePool};

use config::ConfigPlugin;
use level::LevelPlugin;

mod block;
mod block_registry;
mod config;
mod level;
mod overlay;
mod player;
mod position;

#[derive(Resource, Deref, DerefMut)]
struct Database(SqlitePool);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite://chunks.sqlite?mode=rwc").await?;
    let mut conn = pool.acquire().await?;
    init_db(&mut conn).await;
    drop(conn);

    App::new()
        .insert_resource(Database(pool))
        .add_plugins(ConfigPlugin)
        .add_plugins(LevelPlugin)
        .run();

    Ok(())
}

async fn init_db(conn: &mut SqliteConnection) {
    sqlx::query!(
        r#"
            CREATE TABLE IF NOT EXISTS chunks (
                x INTEGER,
                y INTEGER,
                z INTEGER,
                data BLOB,
                UNIQUE(x, y, z)
            )
        "#
    )
    .execute(conn)
    .await;
}
