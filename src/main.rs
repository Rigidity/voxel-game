#![allow(clippy::type_complexity)]

use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};
use bevy_fps_counter::FpsCounterPlugin;
use bevy_mod_mipmap_generator::{generate_mipmaps, MipmapGeneratorPlugin, MipmapGeneratorSettings};
use bevy_rapier3d::prelude::*;

use block::DirtBlock;
use block_registry::BlockRegistry;
use level::Level;
use level_gen::LevelGenPlugin;
use player::PlayerPlugin;

mod block;
mod block_registry;
mod chunk;
mod chunk_builder;
mod level;
mod level_gen;
mod player;
mod position;

fn main() {
    App::new()
        .init_resource::<BlockRegistry>()
        .init_resource::<Level>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .insert_resource(MipmapGeneratorSettings {
            anisotropic_filtering: 1,
            ..default()
        })
        .insert_resource(Msaa::Sample8)
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MipmapGeneratorPlugin,
            TemporalAntiAliasPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            FpsCounterPlugin,
            PlayerPlugin,
            LevelGenPlugin,
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81 * 3.0,
            ..default()
        })
        .add_systems(Startup, (setup_world, register_blocks))
        .add_systems(Update, generate_mipmaps::<StandardMaterial>)
        .run();
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

fn register_blocks(mut registry: ResMut<BlockRegistry>) {
    registry.register::<DirtBlock>("dirt".to_string());
}
