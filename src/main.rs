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
use overlay::OverlayPlugin;
use player::PlayerPlugin;
use rusqlite::Connection;

mod block;
mod block_registry;
mod config;
mod level;
mod overlay;
mod player;
mod position;

#[derive(Resource, Default)]
pub struct ChunkMaterial {
    pub handle: Handle<StandardMaterial>,
}

fn main() {
    App::new()
        .init_resource::<ChunkMaterial>()
        .init_resource::<SharedBlockRegistry>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 1.0)))
        .insert_resource(AmbientLight {
            brightness: 0.8,
            ..default()
        })
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81 * 2.5,
            ..default()
        })
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
        .add_plugins(PlayerPlugin)
        .add_plugins(OverlayPlugin)
        .add_plugins(LevelGenPlugin)
        .add_systems(
            Startup,
            (setup_handles, setup_level, setup_world, register_blocks),
        )
        .run();
}

fn setup_handles(
    mut chunk_material: ResMut<ChunkMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    let texture_handle = server.load("blocks/dirt.png");

    let material = StandardMaterial {
        base_color_texture: Some(texture_handle),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    };

    chunk_material.handle = materials.add(material);
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
            illuminance: 15000.0,
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
