use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use self::chunk_borders::LineMaterial;
use self::info::{display_debug_info, toggle_debug_info, UiState};

mod chunk_borders;
mod info;
mod wireframe;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<chunk_borders::ChunkBordersState>()
            .add_plugin(EguiPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(EntityCountDiagnosticsPlugin)
            .add_plugin(WireframePlugin)
            .add_plugin(MaterialPlugin::<LineMaterial>::default())
            .add_plugin(WorldInspectorPlugin::new())
            .add_system(display_debug_info.run_if(resource_equals(UiState {
                show_debug_info: true,
            })))
            .add_system(toggle_debug_info)
            .add_system(wireframe::toggle)
            .add_system(chunk_borders::toggle);
    }
}
