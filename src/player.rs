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
    chunk::{Dirty, CHUNK_SIZE},
    level::Level,
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
    mut gizmos: Gizmos,
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mouse: Res<Input<MouseButton>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    chunk_query: Query<(Entity, &ChunkPos)>,
) {
    let transform = camera.single();

    let filter = QueryFilter::exclude_dynamic();

    if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(
        transform.translation(),
        transform.forward(),
        Real::from(4.0),
        true,
        filter,
    ) {
        gizmos.ray(transform.translation(), transform.forward(), Color::WHITE);
        gizmos.sphere(intersection.point, Quat::default(), 0.1, Color::RED);
    }

    if mouse.just_pressed(MouseButton::Left) {
        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            transform.translation(),
            transform.forward(),
            Real::from(4.0),
            true,
            filter,
        ) {
            if let Ok((_, chunk_pos)) = chunk_query.get(entity) {
                if let Some(chunk) = level.chunk_mut(chunk_pos) {
                    let (x, y, z) = (
                        intersection.point.x.floor() as i32 - chunk_pos.x * CHUNK_SIZE as i32,
                        intersection.point.y.floor() as i32 - chunk_pos.y * CHUNK_SIZE as i32,
                        intersection.point.z.floor() as i32 - chunk_pos.z * CHUNK_SIZE as i32,
                    );
                    let (x, y, z) = (x as usize, y as usize, z as usize);

                    dbg!(x, y, z);

                    let bound = 0..CHUNK_SIZE;
                    if bound.contains(&x) && bound.contains(&y) && bound.contains(&z) {
                        *chunk.block_relative_mut(x, y, z) = false;
                    }

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
        }
    }
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
