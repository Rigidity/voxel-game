#![allow(clippy::type_complexity)]

use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
};
use bevy_fps_counter::FpsCounterPlugin;
use bevy_rapier3d::prelude::*;

use block::{BasicBlock, Block};
use chunk::{Dirty, CHUNK_SIZE};
use chunk_builder::ChunkBuilder;
use level::Level;
use player::{Player, PlayerCamera, PlayerPlugin};
use position::{BlockPos, ChunkPos};

mod block;
mod chunk;
mod chunk_builder;
mod level;
mod player;
mod position;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .init_resource::<Level>()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TemporalAntiAliasPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            FpsCounterPlugin,
            PlayerPlugin,
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81 * 2.5,
            ..default()
        })
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
    let handle = server.load("dirt.png");

    let width = 8;
    let height = 6;
    let depth = 8;

    for x in -width..width {
        for y in 0..=height {
            for z in -depth..depth {
                let position = ChunkPos::new(x, y, z);
                level.load_chunk(&position);

                let material = StandardMaterial {
                    base_color_texture: Some(handle.clone()),
                    perceptual_roughness: 1.0,
                    reflectance: 0.25,
                    ..default()
                };

                commands.spawn((
                    position,
                    materials.add(material),
                    TransformBundle::from_transform(Transform::from_xyz(
                        (x * CHUNK_SIZE) as f32,
                        (y * CHUNK_SIZE) as f32,
                        (z * CHUNK_SIZE) as f32,
                    )),
                    VisibilityBundle::default(),
                    Friction::new(0.25),
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
            rotation: Quat::from_rotation_x(-FRAC_PI_2),
            ..default()
        },
        ..default()
    });

    commands
        .spawn(Player)
        .insert(TransformBundle::default())
        .insert(Collider::capsule(Vec3::ZERO, Vec3::Y * 1.8, 0.45))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Velocity::default())
        .insert(Friction::new(0.0))
        .insert(Transform::from_xyz(0.0, 90.0, 0.0))
        .with_children(|commands| {
            commands
                .spawn(PlayerCamera)
                .insert(Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(ScreenSpaceAmbientOcclusionBundle::default())
                .insert(TemporalAntiAliasBundle::default());
        });
}

fn generate_meshes(
    mut commands: Commands,
    level: Res<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &ChunkPos), Or<(With<Dirty>, Without<Handle<Mesh>>)>>,
) {
    for (entity, chunk_pos) in query.iter() {
        let (mesh, collider) = build_chunk(&level, chunk_pos);
        let mut entity = commands.entity(entity);
        entity.remove::<Dirty>().insert(meshes.add(mesh));
        if let Some(collider) = collider {
            entity.insert(collider);
        }
    }
}

fn build_chunk(level: &Level, chunk_pos: &ChunkPos) -> (Mesh, Option<Collider>) {
    let mut chunk_builder = ChunkBuilder::new();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let block_pos = BlockPos::from(chunk_pos.clone()) + BlockPos::new(x, y, z);
                if !level.block(&block_pos) {
                    continue;
                }
                let translation = Vec3::new(x as f32, y as f32, z as f32);
                BasicBlock::render(level, &mut chunk_builder, &block_pos, translation);
            }
        }
    }
    chunk_builder.build()
}
