//! Handles grabbing the cursor for the window.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};

pub fn grab(window: &mut Window) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
}

pub fn release(window: &mut Window) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

pub fn toggle_grab(window: &mut Window) {
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => grab(window),
        _ => release(window),
    }
}

pub fn setup(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = primary_window.get_single_mut().unwrap();
    grab(&mut window);
}

pub fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = primary_window.get_single_mut().unwrap();

    for key in keys.get_just_pressed() {
        if key == &KeyCode::Escape {
            toggle_grab(&mut window);
        }
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing cursor plugin");
        app.add_systems(Startup, (setup,))
            .add_systems(Update, (handle_input,));
    }
}
