#![allow(clippy::type_complexity)]

use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};
use bevy_fps_counter::FpsCounterPlugin;
use bevy_rapier3d::prelude::*;

use block_registry::{Block, SharedBlockRegistry};
use chunk_builder::{AdjacentBlocks, ChunkBuilder};
use level::Level;
use level_gen::LevelGenPlugin;
use player::PlayerPlugin;

mod block_registry;
mod chunk;
mod chunk_builder;
mod level;
mod level_gen;
mod player;
mod position;

fn main() {
    App::new()
        .init_resource::<SharedBlockRegistry>()
        .init_resource::<Level>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .insert_resource(Msaa::Sample8)
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
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

fn register_blocks(registry: Res<SharedBlockRegistry>) {
    registry.write().unwrap().register(
        "dirt".to_string(),
        Block {
            render: render_dirt,
        },
    );
}

fn render_dirt(chunk: &mut ChunkBuilder, adjacent: AdjacentBlocks, Vec3 { x, y, z }: Vec3) {
    // Left
    if !adjacent.left {
        let a = chunk.vertex([x, y, z], [-1.0, 0.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x, y + 1.0, z], [-1.0, 0.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x, y + 1.0, z + 1.0], [-1.0, 0.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x, y, z + 1.0], [-1.0, 0.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);
    }

    // Right
    if !adjacent.right {
        let a = chunk.vertex([x + 1.0, y, z], [1.0, 0.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 1.0, y + 1.0, z], [1.0, 0.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 0.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x + 1.0, y, z + 1.0], [1.0, 0.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, b, c, c, d, a]);
    }

    // Top
    if !adjacent.top {
        let a = chunk.vertex([x, y + 1.0, z], [0.0, 1.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 1.0, y + 1.0, z], [0.0, 1.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 1.0, y + 1.0, z + 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x, y + 1.0, z + 1.0], [0.0, 1.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);
    }

    // Bottom
    if !adjacent.bottom {
        let a = chunk.vertex([x, y, z], [0.0, -1.0, 0.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 1.0, y, z], [0.0, -1.0, 0.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 1.0, y, z + 1.0], [0.0, -1.0, 0.0], [1.0, 1.0]);
        let d = chunk.vertex([x, y, z + 1.0], [0.0, -1.0, 0.0], [1.0, 0.0]);
        chunk.indices([a, b, c, c, d, a]);
    }

    // Front
    if !adjacent.front {
        let a = chunk.vertex([x, y, z + 1.0], [0.0, 0.0, 1.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 1.0, y, z + 1.0], [0.0, 0.0, 1.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 1.0, y + 1.0, z + 1.0], [0.0, 0.0, 1.0], [1.0, 1.0]);
        let d = chunk.vertex([x, y + 1.0, z + 1.0], [0.0, 0.0, 1.0], [1.0, 0.0]);
        chunk.indices([a, b, c, c, d, a]);
    }

    // Back
    if !adjacent.back {
        let a = chunk.vertex([x, y, z], [0.0, 0.0, -1.0], [0.0, 0.0]);
        let b = chunk.vertex([x + 1.0, y, z], [0.0, 0.0, -1.0], [0.0, 1.0]);
        let c = chunk.vertex([x + 1.0, y + 1.0, z], [0.0, 0.0, -1.0], [1.0, 1.0]);
        let d = chunk.vertex([x, y + 1.0, z], [0.0, 0.0, -1.0], [1.0, 0.0]);
        chunk.indices([a, d, c, c, b, a]);
    }
}
