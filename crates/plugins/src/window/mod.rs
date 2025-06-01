use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};

use crate::camera::events::CameraEvent;

pub fn setup(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

pub fn focus(window: &mut Window, camera_events: &mut EventWriter<CameraEvent>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
    camera_events.write(CameraEvent::EnableControls);
}

pub fn unfocus(window: &mut Window, camera_events: &mut EventWriter<CameraEvent>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
    camera_events.write(CameraEvent::DisableControls);
}
pub fn handle_mouse_input(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_events: EventWriter<CameraEvent>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        focus(&mut window, &mut camera_events);
    }
}

pub fn handle_keyboard_input(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_events: EventWriter<CameraEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match window.cursor_options.grab_mode {
            CursorGrabMode::None => {
                focus(&mut window, &mut camera_events);
            }
            _ => {
                unfocus(&mut window, &mut camera_events);
            }
        }
    }
}

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                handle_keyboard_input.run_if(not(egui_wants_any_keyboard_input)),
                handle_mouse_input.run_if(not(egui_wants_any_pointer_input)),
            ),
        );
    }
}
