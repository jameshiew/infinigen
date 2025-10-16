use bevy::prelude::*;

use super::FpsController;

#[derive(Message)]
pub enum CameraMessage {
    EnableControls,
    DisableControls,
}

pub fn handle_camera_messages(
    mut camera_messages: MessageReader<CameraMessage>,
    mut fps_controller: Single<&mut FpsController>,
) {
    for msg in camera_messages.read() {
        match msg {
            CameraMessage::EnableControls => fps_controller.enabled = true,
            CameraMessage::DisableControls => fps_controller.enabled = false,
        }
    }
}
