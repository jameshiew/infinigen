use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};

use crate::scene::{self, LoadedChunk};
use infinigen_common::chunks::CHUNK_SIZE_F32;

#[allow(clippy::too_many_arguments)]
pub fn display_debug_info(
    mut egui: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut camera: Query<(&Transform, &mut crate::camera::settings::Settings)>,
    scene_view: Res<scene::SceneView>,
    scene_zoom: Res<scene::SceneZoom>,
    load_ops: Res<scene::LoadOps>,
    mut update_evs: EventWriter<scene::UpdateSettingsEvent>,
    mut reload_evs: EventWriter<scene::ReloadAllChunksEvent>,
    loaded_chunks: Query<&LoadedChunk>,
) {
    let (camera_wpos, mut camera) = camera.single_mut();
    egui::Window::new("Performance").show(egui.ctx_mut(), |ui| {
        ui.label(format!(
            "FPS: {:.0}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!(
            "Entities: {:.0}",
            diagnostics
                .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!("# queued chunk ops: {}", load_ops.deque.len()));
        ui.label(format!(
            "# non-empty chunks loaded: {}",
            loaded_chunks.iter().count(),
        ));
        if ui.button("Clear and reload all chunks").clicked() {
            reload_evs.send(scene::ReloadAllChunksEvent);
        }
    });

    egui::Window::new("Scene").show(egui.ctx_mut(), |ui| {
        ui.label("Position");
        ui.label(format!("X: {:.2}", camera_wpos.translation.x));
        ui.label(format!("Y: {:.2}", camera_wpos.translation.y));
        ui.label(format!("Z: {:.2}", camera_wpos.translation.z));
        ui.label("Rotation");
        ui.label(format!("X: {:.2}", camera_wpos.rotation.x));
        ui.label(format!("Y: {:.2}", camera_wpos.rotation.y));
        ui.label(format!("Z: {:.2}", camera_wpos.rotation.z));
        ui.label(format!("W: {:.2}", camera_wpos.rotation.w));

        let chunk_pos = [
            (camera_wpos.translation.x / CHUNK_SIZE_F32).floor() as i32,
            (camera_wpos.translation.y / CHUNK_SIZE_F32).floor() as i32,
            (camera_wpos.translation.z / CHUNK_SIZE_F32).floor() as i32,
        ];
        ui.label(format!("Chunk: {:?}", chunk_pos));

        let block_pos = [
            camera_wpos.translation.x.floor() as i32,
            camera_wpos.translation.y.floor() as i32,
            camera_wpos.translation.z.floor() as i32,
        ];
        ui.label(format!("Block: {:?}", block_pos));

        let (mut hview_distance, mut vview_distance) =
            (scene_view.hview_distance, scene_view.vview_distance);
        ui.label("Horizontal view distance");
        if ui.add(Slider::new(&mut hview_distance, 1..=64)).changed()
            && hview_distance != scene_view.hview_distance
        {
            update_evs.send(scene::UpdateSettingsEvent::HorizontalViewDistance(
                hview_distance,
            ));
        };
        ui.label("Vertical view distance");
        if ui.add(Slider::new(&mut vview_distance, 1..=64)).changed()
            && vview_distance != scene_view.vview_distance
        {
            update_evs.send(scene::UpdateSettingsEvent::VerticalViewDistance(
                vview_distance,
            ));
        };

        let mut zoom_level = scene_zoom.zoom_level;
        ui.label("Zoom level");
        if ui.add(Slider::new(&mut zoom_level, -5..=5)).changed()
            && scene_zoom.zoom_level != zoom_level
        {
            update_evs.send(scene::UpdateSettingsEvent::ZoomLevel(zoom_level));
        };

        ui.label("Camera speed");
        ui.add(Slider::new(&mut camera.speed, 1.0..=100.0));
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

pub fn toggle_debug_info(keys: Res<ButtonInput<KeyCode>>, mut ui_state: ResMut<UiState>) {
    for key in keys.get_just_pressed() {
        if key == &KeyCode::F7 {
            ui_state.show_debug_info = !ui_state.show_debug_info;
            tracing::info!(%ui_state.show_debug_info, "Debug UI toggled");
        }
    }
}
