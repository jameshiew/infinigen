//! Handles grabbing the cursor for the window.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};

use crate::AppState;
use crate::camera::events::CameraEvent;

pub fn setup(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    primary_window.cursor_options.grab_mode = CursorGrabMode::Confined;
    primary_window.cursor_options.visible = false;
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    mut camera_events: EventWriter<CameraEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match primary_window.cursor_options.grab_mode {
            CursorGrabMode::None => {
                primary_window.cursor_options.grab_mode = CursorGrabMode::Confined;
                primary_window.cursor_options.visible = false;
                camera_events.write(CameraEvent::EnableControls);
            }
            _ => {
                primary_window.cursor_options.grab_mode = CursorGrabMode::None;
                primary_window.cursor_options.visible = true;
                camera_events.write(CameraEvent::DisableControls);
            }
        }
    }
}

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainGame), setup)
            .add_systems(Update, handle_input);
    }
}
