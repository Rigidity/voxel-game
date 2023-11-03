use bevy::prelude::*;
use block_registry::BlockRegistry;

use config::ConfigPlugin;
use level::LevelPlugin;

mod block;
mod block_registry;
mod config;
mod level;
mod overlay;
mod player;
mod position;

fn main() {
    App::new()
        .init_resource::<BlockRegistry>()
        .add_plugins(ConfigPlugin)
        .add_plugins(LevelPlugin)
        .run();
}
