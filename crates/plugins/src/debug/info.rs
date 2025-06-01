use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Slider};
use infinigen_common::chunks::CHUNK_SIZE_F32;
use leafwing_input_manager::prelude::*;
use smooth_bevy_cameras::LookTransform;
use smooth_bevy_cameras::controllers::fps::FpsCameraController;

use crate::scene::{self, LoadedChunk};

#[allow(clippy::too_many_arguments)]
pub fn display_debug_info(
    mut egui: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    camera_wpos: Single<&Transform, With<Camera>>,
    mut fps_camera_controller: Single<&mut FpsCameraController>,
    look_transform: Single<&LookTransform>,
    scene_view: Res<scene::SceneView>,
    scene_zoom: Res<scene::SceneZoom>,
    chunk_requests: Res<scene::ChunkRequests>,
    mut update_evs: EventWriter<scene::UpdateSettingsEvent>,
    mut reload_evs: EventWriter<scene::ReloadAllChunksEvent>,
    loaded_chunks: Query<&LoadedChunk>,
) {
    egui::Window::new("Position").show(egui.ctx_mut(), |ui| {
        egui::Grid::new("position").show(ui, |ui| {
            ui.label("");
            ui.label("x");
            ui.label("y");
            ui.label("z");
            ui.end_row();

            // Block position
            let block_pos = [
                camera_wpos.translation.x.floor() as i32,
                camera_wpos.translation.y.floor() as i32,
                camera_wpos.translation.z.floor() as i32,
            ];
            ui.label("Block:");
            ui.label(format!("{}", block_pos[0]));
            ui.label(format!("{}", block_pos[1]));
            ui.label(format!("{}", block_pos[2]));
            ui.end_row();

            // Chunk position
            let chunk_pos = [
                (camera_wpos.translation.x / CHUNK_SIZE_F32).floor() as i32,
                (camera_wpos.translation.y / CHUNK_SIZE_F32).floor() as i32,
                (camera_wpos.translation.z / CHUNK_SIZE_F32).floor() as i32,
            ];
            ui.label("Chunk:");
            ui.label(format!("{}", chunk_pos[0]));
            ui.label(format!("{}", chunk_pos[1]));
            ui.label(format!("{}", chunk_pos[2]));
            ui.end_row();

            // Target position
            let target = [
                look_transform.target.x.floor() as i32,
                look_transform.target.y.floor() as i32,
                look_transform.target.z.floor() as i32,
            ];
            ui.label("Looking at:");
            ui.label(format!("{}", target[0]));
            ui.label(format!("{}", target[1]));
            ui.label(format!("{}", target[2]));
            ui.end_row();
        });
    });

    egui::Window::new("Controls").show(egui.ctx_mut(), |ui| {
        {
            let (mut hview_distance, mut vview_distance) =
                (scene_view.hview_distance, scene_view.vview_distance);

            ui.label("Horizontal view distance");
            if ui.add(Slider::new(&mut hview_distance, 1..=64)).changed()
                && hview_distance != scene_view.hview_distance
            {
                update_evs.write(scene::UpdateSettingsEvent::HorizontalViewDistance(
                    hview_distance,
                ));
            };

            ui.label("Vertical view distance");
            if ui.add(Slider::new(&mut vview_distance, 1..=64)).changed()
                && vview_distance != scene_view.vview_distance
            {
                update_evs.write(scene::UpdateSettingsEvent::VerticalViewDistance(
                    vview_distance,
                ));
            };
        }

        {
            let mut zoom_level = scene_zoom.zoom_level;
            ui.label("Zoom level");
            if ui.add(Slider::new(&mut zoom_level, -5..=5)).changed()
                && scene_zoom.zoom_level != zoom_level
            {
                update_evs.write(scene::UpdateSettingsEvent::ZoomLevel(zoom_level));
            };
        }

        ui.label("Camera speed");
        ui.add(Slider::new(
            &mut fps_camera_controller.translate_sensitivity,
            1.0..=100.0,
        ));
        if ui.button("Clear and reload all chunks").clicked() {
            reload_evs.write(scene::ReloadAllChunksEvent);
        }
    });

    egui::Window::new("Stats").show(egui.ctx_mut(), |ui| {
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
