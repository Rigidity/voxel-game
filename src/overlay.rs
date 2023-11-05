use std::time::Duration;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[derive(Resource)]
struct Fps {
    timer: Timer,
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
struct FpsText;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fps>()
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update_fps);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(8.0),
                    top: Val::Px(8.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: String::new(),
                            style: TextStyle {
                                font_size: 26.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        }],
                        ..default()
                    },
                    ..default()
                })
                .insert(FpsText);
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::new(asset_server.load("crosshair.png")),
                style: Style {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ..default()
            });
        });
}

fn update_fps(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut fps: ResMut<Fps>,
    mut text_query: Query<&mut Text, With<FpsText>>,
) {
    if !fps.timer.tick(time.delta()).just_finished() {
        return;
    }

    if fps.timer.paused() {
        for mut text in text_query.iter_mut() {
            let value: &mut String = &mut text.sections[0].value;
            value.clear();
        }
    } else {
        let fps_dialog: Option<f64> = diagnostics
            .get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.average());

        for mut text in text_query.iter_mut() {
            let value: &mut String = &mut text.sections[0].value;

            if let Some(fps) = fps_dialog {
                *value = format!("FPS: {:.0}", fps);
            } else {
                value.clear();
            }
        }
    }
}
