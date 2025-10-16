use bevy::prelude::*;

use super::FpsController;

#[derive(Message)]
pub enum CameraEvent {
    EnableControls,
    DisableControls,
}

pub fn handle_camera_events(
    mut camera_events: MessageReader<CameraEvent>,
    mut fps_controller: Single<&mut FpsController>,
) {
    for ev in camera_events.read() {
        match ev {
            CameraEvent::EnableControls => fps_controller.enabled = true,
            CameraEvent::DisableControls => fps_controller.enabled = false,
        }
    }
}
