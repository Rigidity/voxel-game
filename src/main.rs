#![allow(clippy::type_complexity)]

use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::{CascadeShadowConfigBuilder, ScreenSpaceAmbientOcclusionBundle},
    prelude::*,
};
use bevy_flycam::prelude::*;
use block::{BasicBlock, Block, BlockPos};
use chunk::{Chunk, ChunkPos, Dirty};
use chunk_builder::ChunkBuilder;
use level::Level;

mod block;
mod chunk;
mod chunk_builder;
mod level;

#[derive(Component)]
pub struct GameCamera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.5,
            ..default()
        })
        .insert_resource(Level::default())
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TemporalAntiAliasPlugin,
            NoCameraPlayerPlugin,
        ))
        .add_systems(Startup, setup_world)
        .add_systems(Update, generate_meshes)
        .run();
}

fn setup_world(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    for x in -2i32..2i32 {
        for y in -2i32..2i32 {
            for z in -2i32..2i32 {
                let position = ChunkPos::new(x, y, z);
                level.load_chunk(position);

                let material = StandardMaterial {
                    base_color_texture: Some(server.load("dirt.png")),
                    perceptual_roughness: 1.0,
                    reflectance: 0.0,
                    ..default()
                };

                commands.spawn((
                    position,
                    materials.add(material),
                    TransformBundle::from_transform(Transform::from_xyz(
                        x as f32 * 16.0,
                        y as f32 * 16.0,
                        z as f32 * 16.0,
                    )),
                    VisibilityBundle::default(),
                ));
            }
        }
    }

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 500.0, 0.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_2 * 0.8),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });

    commands
        .spawn((
            GameCamera,
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 32.0, 0.0),
                ..default()
            },
            FlyCam,
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());
}

fn generate_meshes(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &ChunkPos), Or<(With<Dirty>, Without<Handle<Mesh>>)>>,
) {
    for (entity, &position) in query.iter() {
        let chunk = level.load_chunk(position);
        commands
            .entity(entity)
            .remove::<Dirty>()
            .insert(meshes.add(create_mesh(chunk)));
    }
}

fn create_mesh(chunk: &Chunk) -> Mesh {
    let mut chunk_builder = ChunkBuilder::new();
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let block_pos = BlockPos::new(x, y, z);
                if !chunk.get_block(block_pos) {
                    continue;
                }
                BasicBlock::render(&mut chunk_builder, block_pos);
            }
        }
    }
    chunk_builder.mesh()
}
