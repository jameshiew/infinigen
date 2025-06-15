use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};
use leafwing_input_manager::prelude::*;

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
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut camera_events: EventWriter<CameraEvent>,
) {
    if action_state.just_pressed(&Action::ToggleFocus) {
        match window.cursor_options.grab_mode {
            CursorGrabMode::None => {
                focus(&mut window, &mut camera_events);
            }
            _ => {
                unfocus(&mut window, &mut camera_events);
            }
        }
    }

    if action_state.just_pressed(&Action::ForceFocus) {
        focus(&mut window, &mut camera_events);
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
