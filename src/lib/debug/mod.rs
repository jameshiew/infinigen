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
        tracing::info!("Initializing debug UI plugin");
        let application = app.init_resource::<UiState>();
        application.init_resource::<chunk_borders::ChunkBordersState>();

        let plugins = (
            EguiPlugin,
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            WireframePlugin,
            WorldInspectorPlugin::new(),
            MaterialPlugin::<LineMaterial>::default(),
        );
        let systems = (
            display_debug_info.run_if(resource_equals(UiState {
                show_debug_info: true,
            })),
            toggle_debug_info,
            wireframe::toggle,
            chunk_borders::toggle,
        );
        application
            .add_plugins(plugins)
            .add_systems(Update, systems);
    }
}
