use bevy::prelude::*;
use bevy_flycam::{KeyBindings, MovementSettings, NoCameraPlayerPlugin};

use crate::AppState;

pub mod setup;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing camera plugin");
        app.add_systems(OnEnter(AppState::MainGame), (setup::setup,))
            .add_plugins(NoCameraPlayerPlugin)
            .insert_resource(MovementSettings {
                sensitivity: 0.0001,
                speed: 50.0,
            })
            .insert_resource(KeyBindings {
                move_ascend: KeyCode::Space,
                move_descend: KeyCode::ShiftLeft,
                move_forward: KeyCode::KeyW,
                move_backward: KeyCode::KeyS,
                move_left: KeyCode::KeyA,
                move_right: KeyCode::KeyD,
                toggle_grab_cursor: KeyCode::Escape,
            });
    }
}
