#![allow(clippy::type_complexity)]

use bevy::{app::AppExit, prelude::*, utils::HashMap};
use bevy_flycam::prelude::*;
use block::{BasicBlock, Block};
use chunk_builder::ChunkBuilder;
use noise::{NoiseFn, Perlin};

mod block;
mod chunk_builder;

#[derive(Component)]
pub struct GameCamera;

#[derive(Resource, Default)]
pub struct Level {
    loaded_chunks: HashMap<ChunkPos, Chunk>,
}

#[derive(Default)]
pub struct Chunk {
    blocks: [[[bool; 16]; 16]; 16],
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(i32, i32, i32);

#[derive(Component)]
pub struct Dirty;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Level::default())
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup_world)
        .add_systems(Update, (keyboard_input, generate_meshes))
        .run();
}

fn setup_world(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    let noise = Perlin::new(42);

    for i in -2i32..2i32 {
        for j in -2i32..2i32 {
            for k in -2i32..2i32 {
                let mut chunk = Chunk::default();

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let total_x = i * 16 + x as i32;
                            let total_y = j * 16 + y as i32;
                            let total_z = k * 16 + z as i32;

                            chunk.blocks[x][y][z] = (total_y as f64)
                                < (16.0
                                    + 12.0
                                        * noise
                                            .get([total_x as f64 / 16.0, total_z as f64 / 16.0]));
                        }
                    }
                }

                let position = ChunkPos(i, j, k);

                level.loaded_chunks.insert(position, chunk);

                commands.spawn((
                    position,
                    materials.add(server.load("dirt.png").into()),
                    TransformBundle::from_transform(Transform::from_xyz(
                        i as f32 * 16.0,
                        j as f32 * 16.0,
                        k as f32 * 16.0,
                    )),
                    VisibilityBundle::default(),
                ));
            }
        }
    }

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 25000.0,
            ..default()
        },
        ..default()
    });

    commands.spawn((
        GameCamera,
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 32.0, 0.0),
            ..default()
        },
        FlyCam,
    ));
}

fn generate_meshes(
    mut commands: Commands,
    level: Res<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &ChunkPos), Or<(With<Dirty>, Without<Handle<Mesh>>)>>,
) {
    for (entity, position) in query.iter() {
        let chunk = &level.loaded_chunks[position];
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
                if !chunk.blocks[x][y][z] {
                    continue;
                }
                BasicBlock::render(&mut chunk_builder, [x as f32, y as f32, z as f32]);
            }
        }
    }
    chunk_builder.mesh()
}

fn keyboard_input(keys: Res<Input<KeyCode>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
