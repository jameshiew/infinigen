use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::egui::{self, Slider};
use infinigen_common::chunks::CHUNK_SIZE_F32;
use leafwing_input_manager::prelude::*;

use crate::camera::FpsController;
use crate::scene::{self, LoadedChunk};

pub fn display_debug_info(
    mut egui: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    camera_query: Single<(&Transform, &mut FpsController), With<Camera>>,
    scene_view: Res<scene::SceneView>,
    scene_zoom: Res<scene::SceneZoom>,
    chunk_requests: Res<scene::ChunkRequests>,
    mut update_msgs: MessageWriter<scene::UpdateSettingsMessage>,
    mut reload_msgs: MessageWriter<scene::ReloadAllChunksMessage>,
    loaded_chunks: Query<&LoadedChunk>,
) -> Result {
    let (camera_transform, mut fps_controller) = camera_query.into_inner();
    egui::Window::new("Position").show(egui.ctx_mut()?, |ui| {
        egui::Grid::new("position").show(ui, |ui| {
            ui.label("");
            ui.label("x");
            ui.label("y");
            ui.label("z");
            ui.end_row();

            // Block position
            let block_pos = [
                camera_transform.translation.x.floor() as i32,
                camera_transform.translation.y.floor() as i32,
                camera_transform.translation.z.floor() as i32,
            ];
            ui.label("Block:");
            ui.label(format!("{}", block_pos[0]));
            ui.label(format!("{}", block_pos[1]));
            ui.label(format!("{}", block_pos[2]));
            ui.end_row();

            // Chunk position
            let chunk_pos = [
                (camera_transform.translation.x / CHUNK_SIZE_F32).floor() as i32,
                (camera_transform.translation.y / CHUNK_SIZE_F32).floor() as i32,
                (camera_transform.translation.z / CHUNK_SIZE_F32).floor() as i32,
            ];
            ui.label("Chunk:");
            ui.label(format!("{}", chunk_pos[0]));
            ui.label(format!("{}", chunk_pos[1]));
            ui.label(format!("{}", chunk_pos[2]));
            ui.end_row();
        });
    });

    egui::Window::new("Controls").show(egui.ctx_mut()?, |ui| {
        {
            let (mut horizontal_view_distance, mut vertical_view_distance) = (
                scene_view.horizontal_view_distance,
                scene_view.vertical_view_distance,
            );

            ui.label("Horizontal view distance");
            if ui
                .add(Slider::new(&mut horizontal_view_distance, 1..=64))
                .changed()
                && horizontal_view_distance != scene_view.horizontal_view_distance
            {
                update_msgs.write(scene::UpdateSettingsMessage::HorizontalViewDistance(
                    horizontal_view_distance,
                ));
            };

            ui.label("Vertical view distance");
            if ui
                .add(Slider::new(&mut vertical_view_distance, 1..=64))
                .changed()
                && vertical_view_distance != scene_view.vertical_view_distance
            {
                update_msgs.write(scene::UpdateSettingsMessage::VerticalViewDistance(
                    vertical_view_distance,
                ));
            };
        }

        {
            let mut zoom_level = scene_zoom.zoom_level;
            ui.label("Zoom level");
            if ui.add(Slider::new(&mut zoom_level, -5..=5)).changed()
                && scene_zoom.zoom_level != zoom_level
            {
                update_msgs.write(scene::UpdateSettingsMessage::ZoomLevel(zoom_level));
            };
        }

        ui.label("Camera speed");
        ui.add(Slider::new(&mut fps_controller.movement_speed, 1.0..=100.0));
        if ui.button("Clear and reload all chunks").clicked() {
            reload_msgs.write(scene::ReloadAllChunksMessage);
        }
    });

    egui::Window::new("Stats").show(egui.ctx_mut()?, |ui| {
        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("FPS");
            ui.label(format!(
                "{:.0}",
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .unwrap()
                    .average()
                    .unwrap_or_default()
            ));
            ui.end_row();

            ui.label("Entities");
            ui.label(format!(
                "{:.0}",
                diagnostics
                    .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                    .unwrap()
                    .average()
                    .unwrap_or_default()
            ));
            ui.end_row();

            ui.label("Queued chunk ops");
            ui.label(format!("{}", chunk_requests.len()));
            ui.end_row();

            ui.label("Non-empty chunks loaded");
            ui.label(format!("{}", loaded_chunks.iter().count()));
            ui.end_row();
        });
    });
    Ok(())
}

#[derive(Debug, Resource, Eq, PartialEq)]
pub struct UiState {
    pub show_debug_info: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_debug_info: true,
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    ToggleDebugUI,
}

pub fn setup_actions(mut commands: Commands) {
    // Describes how to convert from player inputs into those actions
    let input_map = InputMap::new([(Action::ToggleDebugUI, KeyCode::F7)]);
    commands.spawn(input_map);
}

pub fn handle_actions(action_state: Single<&ActionState<Action>>, mut ui_state: ResMut<UiState>) {
    if action_state.just_pressed(&Action::ToggleDebugUI) {
        ui_state.show_debug_info = !ui_state.show_debug_info;
        tracing::info!(%ui_state.show_debug_info, "Debug UI toggled");
    }
}
