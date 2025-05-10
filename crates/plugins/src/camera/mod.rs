use bevy::prelude::*;
use smooth_bevy_cameras::LookTransformPlugin;
use smooth_bevy_cameras::controllers::fps::FpsCameraPlugin;

use crate::AppState;

pub mod events;
pub mod setup;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing camera plugin");
        app.add_plugins(LookTransformPlugin)
            .add_plugins(FpsCameraPlugin::default())
            .add_systems(OnEnter(AppState::MainGame), setup::setup)
            .add_systems(Update, events::handle_camera_events)
            .add_event::<events::CameraEvent>();
    }
}
