#![allow(clippy::type_complexity)]

use std::f32::consts::FRAC_PI_2;

use async_io::block_on;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_fps_counter::FpsCounterPlugin;
use bevy_rapier3d::prelude::*;
use futures_lite::future;

use block::{AdjacentBlocks, BasicBlock, Block};
use chunk::{Chunk, Dirty, CHUNK_SIZE};
use chunk_builder::ChunkBuilder;
use level::Level;
use player::{Player, PlayerPlugin};
use position::{BlockPos, ChunkPos};

mod block;
mod chunk;
mod chunk_builder;
mod level;
mod player;
mod position;

#[derive(Resource)]
struct ChunkDistance(usize);

impl Default for ChunkDistance {
    fn default() -> Self {
        Self(8)
    }
}

#[derive(Component)]
pub struct MeshTask(Task<(Mesh, Option<Collider>)>);

fn main() {
    App::new()
        .init_resource::<Level>()
        .init_resource::<ChunkDistance>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TemporalAntiAliasPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            FpsCounterPlugin,
            PlayerPlugin,
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -9.81 * 3.0,
            ..default()
        })
        .add_systems(Startup, setup_world)
        .add_systems(
            Update,
            (
                (load_chunks, unload_chunks),
                apply_deferred,
                (generate_meshes, insert_meshes),
            )
                .chain(),
        )
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

fn load_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level: ResMut<Level>,
    max_distance: Res<ChunkDistance>,
    player: Query<&Transform, With<Player>>,
    server: Res<AssetServer>,
) {
    let handle = server.load("dirt.png");
    let block_distance = (max_distance.0 * CHUNK_SIZE) as f32;
    let transform = player.single();

    let player_chunk_pos = BlockPos::new(
        transform.translation.x as i32,
        transform.translation.y as i32,
        transform.translation.z as i32,
    )
    .chunk_pos()
    .0;
    let player_center_pos = player_chunk_pos.center();

    let distance = max_distance.0 as i32;

    for x in -distance..=distance {
        for y in -distance..=distance {
            for z in -distance..=distance {
                let chunk_pos = player_chunk_pos.clone() + ChunkPos::new(x, y, z);

                if level.get_chunk(&chunk_pos).is_some() {
                    continue;
                }

                if player_center_pos.distance(chunk_pos.center()) <= block_distance {
                    level.load_chunk(&chunk_pos);

                    let material = StandardMaterial {
                        base_color_texture: Some(handle.clone()),
                        perceptual_roughness: 1.0,
                        reflectance: 0.25,
                        ..default()
                    };

                    commands.spawn((
                        chunk_pos.clone(),
                        materials.add(material),
                        TransformBundle::from_transform(Transform::from_xyz(
                            (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                        )),
                        VisibilityBundle::default(),
                        Friction::new(0.25),
                        Dirty,
                    ));
                }
            }
        }
    }
}

fn unload_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    max_distance: Res<ChunkDistance>,
    chunks: Query<(Entity, &ChunkPos)>,
    player: Query<&Transform, With<Player>>,
) {
    let max_distance = (max_distance.0 * CHUNK_SIZE) as f32;
    let transform = player.single();
    let player_chunk_pos = BlockPos::new(
        transform.translation.x as i32,
        transform.translation.y as i32,
        transform.translation.z as i32,
    )
    .chunk_pos()
    .0;
    let player_center_pos = player_chunk_pos.center();

    for (chunk, chunk_pos) in chunks.iter() {
        if player_center_pos.distance(chunk_pos.center()) > max_distance {
            commands.entity(chunk).despawn_recursive();
            level.unload_chunk(chunk_pos);
        }
    }
}

fn insert_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut MeshTask)>,
) {
    for (entity, mut mesh_task) in query.iter_mut() {
        if let Some((mesh, collider)) = block_on(future::poll_once(&mut mesh_task.0)) {
            let mut entity = commands.entity(entity);

            entity.remove::<MeshTask>();
            entity.insert(meshes.add(mesh));

            if let Some(collider) = collider {
                entity.insert(collider);
            }
        }
    }
}

fn generate_meshes(
    mut commands: Commands,
    level: Res<Level>,
    query: Query<(Entity, &ChunkPos), With<Dirty>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, chunk_pos) in query.iter() {
        let Some(chunk) = level.get_chunk(chunk_pos) else {
            continue;
        };

        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        let left = level
            .get_chunk(&(chunk_pos.clone() - ChunkPos::X))
            .map(|chunk| {
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (y, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = chunk.block_relative(CHUNK_SIZE - 1, y, z);
                    }
                }
                data
            });

        let right = level
            .get_chunk(&(chunk_pos.clone() + ChunkPos::X))
            .map(|chunk| {
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (y, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = chunk.block_relative(0, y, z);
                    }
                }
                data
            });

        let top = level
            .get_chunk(&(chunk_pos.clone() + ChunkPos::Y))
            .map(|chunk| {
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = chunk.block_relative(x, 0, z);
                    }
                }
                data
            });

        let bottom = level
            .get_chunk(&(chunk_pos.clone() - ChunkPos::Y))
            .map(|chunk| {
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (z, data) in data.iter_mut().enumerate() {
                        *data = chunk.block_relative(x, CHUNK_SIZE - 1, z);
                    }
                }
                data
            });

        let front = level
            .get_chunk(&(chunk_pos.clone() + ChunkPos::Z))
            .map(|chunk| {
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (y, data) in data.iter_mut().enumerate() {
                        *data = chunk.block_relative(x, y, 0);
                    }
                }
                data
            });

        let back = level
            .get_chunk(&(chunk_pos.clone() - ChunkPos::Z))
            .map(|chunk| {
                let mut data = [[false; CHUNK_SIZE]; CHUNK_SIZE];
                for (x, data) in data.iter_mut().enumerate() {
                    for (y, data) in data.iter_mut().enumerate() {
                        *data = chunk.block_relative(x, y, CHUNK_SIZE - 1);
                    }
                }
                data
            });

        let adjacent = AdjacentChunkData {
            left,
            right,
            top,
            bottom,
            front,
            back,
        };
        let chunk = chunk.clone();

        let task = thread_pool.spawn(async move { build_chunk(adjacent, chunk) });

        entity.remove::<Dirty>().insert(MeshTask(task));
    }
}

pub struct AdjacentChunkData {
    left: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    right: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    top: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    bottom: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    front: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    back: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
}

fn build_chunk(adjacent: AdjacentChunkData, chunk: Chunk) -> (Mesh, Option<Collider>) {
    let mut chunk_builder = ChunkBuilder::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if !chunk.block_relative(x, y, z) {
                    continue;
                }

                let adjacent_sides = AdjacentBlocks {
                    left: if x == 0 {
                        adjacent.left.map(|data| data[y][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x - 1, y, z)
                    },
                    right: if x == CHUNK_SIZE - 1 {
                        adjacent.right.map(|data| data[y][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x + 1, y, z)
                    },
                    bottom: if y == 0 {
                        adjacent.bottom.map(|data| data[x][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y - 1, z)
                    },
                    top: if y == CHUNK_SIZE - 1 {
                        adjacent.top.map(|data| data[x][z]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y + 1, z)
                    },
                    back: if z == 0 {
                        adjacent.back.map(|data| data[x][y]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y, z - 1)
                    },
                    front: if z == CHUNK_SIZE - 1 {
                        adjacent.front.map(|data| data[x][y]).unwrap_or(false)
                    } else {
                        chunk.block_relative(x, y, z + 1)
                    },
                };

                let translation = Vec3::new(x as f32, y as f32, z as f32);

                BasicBlock::render(&mut chunk_builder, adjacent_sides, translation);
            }
        }
    }

    let mesh = chunk_builder.build();
    let mut collider = None;

    if mesh.count_vertices() > 0 {
        collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh);
    }

    (mesh, collider)
}
