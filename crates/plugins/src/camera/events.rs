use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use super::FpsController;

#[derive(Event)]
pub enum CameraEvent {
    EnableControls,
    DisableControls,
}

pub fn handle_camera_events(
    mut camera_events: EventReader<CameraEvent>,
    mut fps_controller: Single<&mut FpsController>,
    mut windows: Single<&mut Window>,
) {
    for ev in camera_events.read() {
        match ev {
            CameraEvent::EnableControls => {
                fps_controller.enabled = true;
                // Grab cursor when enabling controls
                windows.cursor_options.grab_mode = CursorGrabMode::Locked;
                windows.cursor_options.visible = false;
            }
            CameraEvent::DisableControls => {
                fps_controller.enabled = false;
                // Release cursor when disabling controls
                windows.cursor_options.grab_mode = CursorGrabMode::None;
                windows.cursor_options.visible = true;
            }
        }
    }
}
