use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_inspector_egui::bevy_egui::input::{
    egui_wants_any_keyboard_input, egui_wants_any_pointer_input,
};
use leafwing_input_manager::prelude::*;

use crate::camera::events::CameraEvent;

pub fn setup(mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_cursor_options.visible = false;
}

pub fn focus(
    primary_cursor_options: &mut CursorOptions,
    camera_events: &mut MessageWriter<CameraEvent>,
) {
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_cursor_options.visible = false;
    camera_events.write(CameraEvent::EnableControls);
}

pub fn unfocus(
    primary_cursor_options: &mut CursorOptions,
    camera_events: &mut MessageWriter<CameraEvent>,
) {
    primary_cursor_options.grab_mode = CursorGrabMode::None;
    primary_cursor_options.visible = true;
    camera_events.write(CameraEvent::DisableControls);
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    ToggleFocus,
    ForceFocus,
}

pub fn setup_actions(mut commands: Commands) {
    let mut input_map = InputMap::new([(Action::ToggleFocus, KeyCode::Escape)]);
    input_map.insert(Action::ForceFocus, MouseButton::Left);
    commands.spawn(input_map);
}

pub fn handle_actions(
    action_state: Single<&ActionState<Action>>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera_events: MessageWriter<CameraEvent>,
) {
    if action_state.just_pressed(&Action::ToggleFocus) {
        match primary_cursor_options.grab_mode {
            CursorGrabMode::None => {
                focus(&mut primary_cursor_options, &mut camera_events);
            }
            _ => {
                unfocus(&mut primary_cursor_options, &mut camera_events);
            }
        }
    }

    if action_state.just_pressed(&Action::ForceFocus) {
        focus(&mut primary_cursor_options, &mut camera_events);
    }
}
pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default())
            .add_systems(Startup, (setup, setup_actions))
            .add_systems(
                Update,
                handle_actions.run_if(
                    not(egui_wants_any_keyboard_input).and(not(egui_wants_any_pointer_input)),
                ),
            );
    }
}
