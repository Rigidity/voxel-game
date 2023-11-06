#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};
use bevy_rapier3d::prelude::*;
use block::{dirt::render_dirt, Block};
use block_registry::BlockRegistry;

use config::ConfigPlugin;
use level::LevelPlugin;
use overlay::OverlayPlugin;
use player::PlayerPlugin;

mod block;
mod block_registry;
mod config;
mod level;
mod overlay;
mod player;
mod position;

#[derive(Resource, Default)]
struct ChunkMaterial {
    handle: Handle<StandardMaterial>,
}

fn main() {
    App::new()
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
        .init_resource::<BlockRegistry>()
        .init_resource::<ChunkMaterial>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.8,
            ..default()
        })
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81 * 2.5,
            ..default()
        })
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(ConfigPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(OverlayPlugin)
        .add_systems(Startup, (register_blocks, setup_handle, setup_world))
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

fn setup_handle(
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

fn setup_world(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            illuminance: 10000.0,
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
