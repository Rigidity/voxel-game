use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasBundle,
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use bevy_rapier3d::prelude::*;

use crate::{
    level::{Dirty, Level, CHUNK_SIZE},
    position::{BlockPos, ChunkPos},
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(0.00012)
    }
}

#[derive(Resource)]
pub struct MovementSpeed(pub f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(70.0)
    }
}

#[derive(Resource)]
pub struct MovementControls {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
}

impl Default for MovementControls {
    fn default() -> Self {
        Self {
            forward: KeyCode::W,
            backward: KeyCode::S,
            left: KeyCode::A,
            right: KeyCode::D,
            jump: KeyCode::Space,
        }
    }
}

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MouseSensitivity>()
            .init_resource::<MovementSpeed>()
            .init_resource::<MovementControls>()
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
        .insert(Collider::capsule(Vec3::ZERO, Vec3::Y * 0.9, 0.45))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Velocity::default())
        .insert(Friction::new(0.0))
        .insert(Transform::from_xyz(0.0, 90.0, 0.0))
        .with_children(|commands| {
            commands
                .spawn(PlayerCamera)
                .insert(ScreenSpaceAmbientOcclusionBundle::default())
                .insert(TemporalAntiAliasBundle::default())
                .insert(FogSettings {
                    falloff: FogFalloff::Linear {
                        start: (CHUNK_SIZE * 6) as f32,
                        end: (CHUNK_SIZE * 8) as f32,
                    },
                    ..default()
                })
                .insert(Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: FRAC_PI_2,
                        ..default()
                    }),
                    ..default()
                });
        });
}

fn remove_block(
    mut level: ResMut<Level>,
    mut commands: Commands,
    mut gizmos: Gizmos,
    mouse: Res<Input<MouseButton>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    chunk_query: Query<(Entity, &ChunkPos)>,
) {
    let transform = camera.single();

    if let Ok((x, y, z)) = raycast_blocks(&level, transform.translation(), transform.forward(), 6) {
        if mouse.just_pressed(MouseButton::Left) {
            let (chunk_pos, (rx, ry, rz)) = BlockPos::new(x, y, z).chunk_pos();
            let Some(chunk) = level.chunk_mut(&chunk_pos) else {
                return;
            };

            *chunk.block_mut(rx, ry, rz) = None;

            if let Some(entity) = chunk_query
                .iter()
                .find(|entity| entity.1 == &chunk_pos)
                .map(|entity| entity.0)
            {
                commands.entity(entity).insert(Dirty);

                for adjacent in [
                    chunk_pos.clone() - ChunkPos::X,
                    chunk_pos.clone() + ChunkPos::X,
                    chunk_pos.clone() - ChunkPos::Y,
                    chunk_pos.clone() + ChunkPos::Y,
                    chunk_pos.clone() - ChunkPos::Z,
                    chunk_pos.clone() + ChunkPos::Z,
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
        let (chunk_pos, (rx, ry, rz)) = BlockPos::new(x, y, z).chunk_pos();
        if let Some(chunk) = level.chunk(&chunk_pos) {
            if chunk.block(rx, ry, rz).is_some() {
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
    sensitivity: Res<MouseSensitivity>,
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
        pitch -= (sensitivity.0 * ev.delta.y * window_scale).to_radians();
        yaw -= (sensitivity.0 * ev.delta.x * window_scale).to_radians();

        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);
        let pitch_rotation = Quat::from_axis_angle(Vec3::X, pitch.clamp(-1.54, 1.54));
        transform.rotation = yaw_rotation * pitch_rotation;
    }
}

fn player_move(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    speed: Res<MovementSpeed>,
    controls: Res<MovementControls>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut Velocity, With<Player>>,
) {
    let window = primary_window.single();
    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    };

    let transform = camera.single();
    let mut velocity = player.single_mut();

    let mut movement = Vec3::ZERO;
    let local_z = transform.local_z();
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
    let right = Vec3::new(local_z.z, 0.0, -local_z.x);

    if keyboard.pressed(controls.forward) {
        movement += forward;
    }

    if keyboard.pressed(controls.backward) {
        movement -= forward;
    }

    if keyboard.pressed(controls.left) {
        movement -= right;
    }

    if keyboard.pressed(controls.right) {
        movement += right;
    }

    let slow_factor = (1.0 - time.delta_seconds() * 8.0).max(0.0);
    velocity.linvel.x *= slow_factor;
    velocity.linvel.z *= slow_factor;

    velocity.linvel += movement.normalize_or_zero() * time.delta_seconds() * speed.0;

    if keyboard.just_pressed(controls.jump) {
        velocity.linvel.y = 9.0;
    }
}

fn setup_input(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        set_grab(&mut window, true);
    }
}

fn toggle_grab(
    keys: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            match window.cursor.grab_mode {
                CursorGrabMode::None => set_grab(&mut window, true),
                _ => set_grab(&mut window, false),
            }
        }
    }
}

fn set_grab(window: &mut Window, grab: bool) {
    if grab {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    } else {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
