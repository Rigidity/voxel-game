use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};
use bevy_fps_counter::FpsCounterPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use block::{dirt::render_dirt, Block};
use block_registry::BlockRegistry;

use config::ConfigPlugin;
use level::LevelPlugin;
use player::PlayerPlugin;

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
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Game".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsCounterPlugin)
        .add_plugins(ConfigPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, register_blocks)
        .run();
}

fn register_blocks(registry: Res<BlockRegistry>) {
    registry.write().register(
        "dirt".to_string(),
        Block {
            render: render_dirt,
        },
    );
}
