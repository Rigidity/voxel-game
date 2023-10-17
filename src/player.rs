use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use bevy_rapier3d::prelude::KinematicCharacterController;

use crate::GameState;

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(OnEnter(GameState::InGame), enter_grab_cursor)
            .add_systems(OnExit(GameState::InGame), exit_ungrab_cursor)
            .add_systems(
                Update,
                (escape_toggle_grab).run_if(in_state(GameState::InGame)),
            );
    }
}

const SENSITIVITY: f32 = 0.00012;
const SPEED: f32 = 12.0;

fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut KinematicCharacterController, &Transform), With<Player>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (mut controller, transform) in query.iter_mut() {
            let mut added_velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
            let right = Vec3::new(local_z.z, 0.0, -local_z.x);

            for &key in keys.get_pressed() {
                if window.cursor.grab_mode != CursorGrabMode::None {
                    match key {
                        KeyCode::W => added_velocity += forward,
                        KeyCode::S => added_velocity -= forward,
                        KeyCode::A => added_velocity -= right,
                        KeyCode::D => added_velocity += right,
                        _ => {}
                    }
                }
            }

            if keys.just_pressed(KeyCode::Space) {
                added_velocity.y += 24.0;
            } else {
                added_velocity.y -= 0.4;
            }

            controller.translation = Some(added_velocity * time.delta_seconds() * SPEED);
        }
    }
}

fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            for ev in state.reader_motion.iter(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (SENSITIVITY * ev.delta.y * window_scale).to_radians();
                        yaw -= (SENSITIVITY * ev.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    }
}

fn escape_toggle_grab(
    keys: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            toggle_grab(&mut window);
        }
    }
}

fn enter_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        set_grab(&mut window, true);
    }
}

fn exit_ungrab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        set_grab(&mut window, true);
    }
}

fn toggle_grab(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => set_grab(window, true),
        _ => set_grab(window, false),
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
