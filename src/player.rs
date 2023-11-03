use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::{experimental::taa::TemporalAntiAliasBundle, tonemapping::Tonemapping},
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use bevy_rapier3d::prelude::*;

use crate::{
    config::Config,
    level::{Dirty, Level, CHUNK_SIZE},
    position::{BlockPos, ChunkPos},
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(Startup, (setup_player, setup_input))
            .add_systems(
                Update,
                (
                    toggle_grab,
                    player_look,
                    player_move,
                    remove_block.after(apply_deferred),
                ),
            );
    }
}

fn setup_player(mut commands: Commands) {
    commands
        .spawn(Player)
        .insert(TransformBundle::default())
        .insert(Collider::cuboid(0.4, 0.8, 0.4))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Ccd::enabled())
        .insert(Velocity::default())
        .insert(Transform::from_xyz(0.0, 20.0, 0.0))
        .insert(Friction::new(0.0))
        .with_children(|commands| {
            commands
                .spawn(PlayerCamera)
                .insert(ScreenSpaceAmbientOcclusionBundle::default())
                .insert(TemporalAntiAliasBundle::default())
                .insert(FogSettings {
                    falloff: FogFalloff::Linear {
                        start: (CHUNK_SIZE * 5) as f32,
                        end: (CHUNK_SIZE * 7) as f32,
                    },
                    color: Color::BLACK,
                    ..default()
                })
                .insert(Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 0.7, 0.0),
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: FRAC_PI_2,
                        ..default()
                    }),
                    tonemapping: Tonemapping::None,
                    ..default()
                });
        });
}

fn remove_block(
    mut commands: Commands,
    mut gizmos: Gizmos,
    level: Res<Level>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<Input<MouseButton>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    chunk_query: Query<(Entity, &ChunkPos)>,
) {
    let window = primary_window.single();

    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let transform = camera.single();

    if let Ok((x, y, z)) = raycast_blocks(&level, transform.translation(), transform.forward(), 6) {
        if mouse.just_pressed(MouseButton::Left) {
            let block_pos = BlockPos::new(x, y, z);
            let chunk_pos = ChunkPos::from(block_pos);
            let relative = block_pos.block_in_chunk();
            let Some(chunk) = level.read().chunk(chunk_pos).cloned() else {
                return;
            };

            *chunk.write().block_mut(relative.0, relative.1, relative.2) = None;

            if let Some(entity) = chunk_query
                .iter()
                .find(|entity| entity.1 == &chunk_pos)
                .map(|entity| entity.0)
            {
                commands.entity(entity).insert(Dirty);

                for adjacent in [
                    chunk_pos - ChunkPos::X,
                    chunk_pos + ChunkPos::X,
                    chunk_pos - ChunkPos::Y,
                    chunk_pos + ChunkPos::Y,
                    chunk_pos - ChunkPos::Z,
                    chunk_pos + ChunkPos::Z,
                ] {
                    if let Some((entity, _)) = chunk_query.iter().find(|e| e.1 == &adjacent) {
                        commands.entity(entity).insert(Dirty);
                    }
                }
            }
        }

        gizmos.cuboid(
            Transform::from_xyz(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5)
                .with_scale(Vec3::ONE),
            Color::BLACK,
        );
    }
}

fn raycast_blocks(
    level: &Level,
    start: Vec3,
    direction: Vec3,
    max_distance: i32,
) -> Result<(i32, i32, i32), (i32, i32, i32)> {
    // Start position in the grid
    let mut x = start.x.floor() as i32;
    let mut y = start.y.floor() as i32;
    let mut z = start.z.floor() as i32;

    // Determine the step direction (1 or -1) for x, y, z
    let step_x = if direction.x >= 0.0 { 1 } else { -1 };
    let step_y = if direction.y >= 0.0 { 1 } else { -1 };
    let step_z = if direction.z >= 0.0 { 1 } else { -1 };

    // How far along the ray must we move for each component
    // to cross a block boundary?
    let delta_x = (1.0 / direction.x).abs();
    let delta_y = (1.0 / direction.y).abs();
    let delta_z = (1.0 / direction.z).abs();

    // Initial values
    let mut t_next_x = delta_x;
    let mut t_next_y = delta_y;
    let mut t_next_z = delta_z;

    // Traverse the grid up to max_distance
    for _ in 0..max_distance {
        // Check for a block at the current position
        let block_pos = BlockPos::new(x, y, z);
        let chunk_pos = ChunkPos::from(block_pos);
        let (rx, ry, rz) = block_pos.block_in_chunk();
        if let Some(chunk) = level.read().chunk(chunk_pos) {
            if chunk.read().block(rx, ry, rz).is_some() {
                return Ok((x, y, z));
            }
        };

        // Move ray to the next nearest block boundary in x, y, or z
        if t_next_x < t_next_y && t_next_x < t_next_z {
            x += step_x;
            t_next_x += delta_x;
        } else if t_next_y < t_next_z {
            y += step_y;
            t_next_y += delta_y;
        } else {
            z += step_z;
            t_next_z += delta_z;
        }
    }

    // Ray didn't hit any block within max_distance
    Err((x, y, z))
}

fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    motion: Res<Events<MouseMotion>>,
    config: Res<Config>,
    mut state: ResMut<InputState>,
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
) {
    let window = primary_window.single();
    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    };

    let mut transform = camera.single_mut();

    for ev in state.reader_motion.iter(&motion) {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        let window_scale = window.height().min(window.width());
        pitch -= (config.mouse_sensitivity * ev.delta.y * window_scale).to_radians();
        yaw -= (config.mouse_sensitivity * ev.delta.x * window_scale).to_radians();

        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);
        let pitch_rotation = Quat::from_axis_angle(Vec3::X, pitch.clamp(-1.54, 1.54));
        transform.rotation = yaw_rotation * pitch_rotation;
    }
}

fn player_move(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    config: Res<Config>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut Velocity, With<Player>>,
) {
    let window = primary_window.single();
    let transform = camera.single();
    let mut velocity = player.single_mut();

    if window.cursor.grab_mode != CursorGrabMode::None {
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
        let right = Vec3::new(local_z.z, 0.0, -local_z.x);

        let mut movement = Vec3::ZERO;

        macro_rules! apply {
            ($op:tt $dir:ident if $key:ident) => {
                if keyboard.pressed(config.movement_controls.$key) {
                    movement $op $dir;
                }
            };
        }

        apply!(+= forward if move_forward);
        apply!(-= forward if move_backward);
        apply!(+= right if strafe_right);
        apply!(-= right if strafe_left);

        velocity.linvel +=
            movement.normalize_or_zero() * time.delta_seconds() * config.movement_speed;

        if keyboard.just_pressed(config.movement_controls.jump) {
            velocity.linvel.y = 9.0;
        }
    }

    let slow_factor = (1.0 - time.delta_seconds() * 8.0).max(0.0);
    velocity.linvel.x *= slow_factor;
    velocity.linvel.z *= slow_factor;
}

fn setup_input(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    grab(&mut primary_window.single_mut());
}

fn toggle_grab(
    keys: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    let mut window = primary_window.single_mut();

    if window.cursor.grab_mode == CursorGrabMode::None {
        grab(&mut window);
    } else {
        ungrab(&mut window);
    }
}

fn grab(window: &mut Window) {
    window.cursor.grab_mode = CursorGrabMode::Confined;
    window.cursor.visible = false;
}

fn ungrab(window: &mut Window) {
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
}
