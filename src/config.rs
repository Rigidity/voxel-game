use std::{fs, path::Path};

use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "config.ron";

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_config)
            .add_systems(Update, save_config);
    }
}

fn write_config_file(config: &Config) {
    let pretty = PrettyConfig::new();
    let text = ron::ser::to_string_pretty(&config, pretty).unwrap();
    fs::write(CONFIG_PATH, text).unwrap();
}

fn load_config(mut commands: Commands) {
    let config = if Path::new(CONFIG_PATH).exists() {
        let text = fs::read_to_string(CONFIG_PATH).unwrap();
        ron::from_str(&text).unwrap()
    } else {
        let config = Config::default();
        write_config_file(&config);
        config
    };

    commands.insert_resource(config);
}

fn save_config(config: Res<Config>) {
    if config.is_changed() && !config.is_added() {
        write_config_file(&config);
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub render_distance: i32,
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,
    pub movement_controls: MovementControls,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            render_distance: 8,
            mouse_sensitivity: 0.00012,
            movement_speed: 70.0,
            movement_controls: MovementControls::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementControls {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub strafe_left: KeyCode,
    pub strafe_right: KeyCode,
    pub jump: KeyCode,
    pub descend: KeyCode,
}

impl Default for MovementControls {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::W,
            move_backward: KeyCode::S,
            strafe_left: KeyCode::A,
            strafe_right: KeyCode::D,
            jump: KeyCode::Space,
            descend: KeyCode::ShiftLeft,
        }
    }
}
