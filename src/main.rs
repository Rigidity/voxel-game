#![allow(clippy::type_complexity)]

use std::{
    f32::consts::FRAC_PI_2,
    sync::{Arc, Mutex},
};

use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*, utils::HashMap};
use bevy_fps_counter::FpsCounterPlugin;
use bevy_rapier3d::prelude::*;

use block::{dirt::render_dirt, Block};
use block_registry::SharedBlockRegistry;
use config::ConfigPlugin;
use level::{Level, LevelGenPlugin};
use noise::Perlin;
use player::PlayerPlugin;
use rusqlite::Connection;

mod block;
mod block_registry;
mod config;
mod level;
mod player;
mod position;

fn main() {
    App::new()
        .init_resource::<SharedBlockRegistry>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsCounterPlugin)
        .add_plugins(ConfigPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(LevelGenPlugin)
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81 * 3.0,
            ..default()
        })
        .add_systems(Startup, (setup_level, setup_world, register_blocks))
        .run();
}

fn setup_level(mut commands: Commands) {
    let connection = Connection::open("chunks.sqlite").unwrap();

    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS `chunks` (
        `x` INTEGER,
        `y` INTEGER,
        `z` INTEGER,
        `data` BLOB
    )",
            (),
        )
        .unwrap();

    commands.insert_resource(Level {
        connection: Arc::new(Mutex::new(connection)),
        loaded_chunks: HashMap::new(),
        noise: Perlin::default(),
    });
}

fn setup_world(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 500.0, 0.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_2),
            ..default()
        },
        ..default()
    });
}

fn register_blocks(registry: Res<SharedBlockRegistry>) {
    registry.write().unwrap().register(
        "dirt".to_string(),
        Block {
            render: render_dirt,
        },
    );
}
