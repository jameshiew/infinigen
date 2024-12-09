use bevy::prelude::*;

use self::input::{keyboard, mouse, InputState, KeyBindings};
use crate::AppState;

pub mod input;
pub mod settings;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing camera plugin");
        app.add_systems(OnEnter(AppState::MainGame), (settings::setup,))
            .init_resource::<KeyBindings>()
            .init_resource::<InputState>()
            .add_systems(
                Update,
                (keyboard, mouse).run_if(in_state(AppState::MainGame)),
            );
    }
}
