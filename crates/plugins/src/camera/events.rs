use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::FpsCameraController;

#[derive(Event)]
pub enum CameraEvent {
    EnableControls,
    DisableControls,
}

pub fn handle_camera_events(
    mut camera_events: EventReader<CameraEvent>,
    mut fps_camera_controls: Single<&mut FpsCameraController>,
) {
    for ev in camera_events.read() {
        match ev {
            CameraEvent::EnableControls => fps_camera_controls.enabled = true,
            CameraEvent::DisableControls => fps_camera_controls.enabled = false,
        }
    }
}
