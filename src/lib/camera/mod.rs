use bevy::prelude::*;

pub mod input;
pub mod settings;

use self::input::{keyboard, mouse, InputState, KeyBindings};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (settings::setup,))
            .init_resource::<KeyBindings>()
            .init_resource::<InputState>()
            .add_systems(Update, (keyboard, mouse));
    }
}
