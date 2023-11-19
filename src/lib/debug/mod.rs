use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[cfg(feature = "chunk-borders")]
use self::chunk_borders::LineMaterial;
use self::info::{display_debug_info, toggle_debug_info, UiState};

#[cfg(feature = "chunk-borders")]
mod chunk_borders;
mod info;
mod wireframe;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing debug UI plugin");
        let application = app.init_resource::<UiState>();
        #[cfg(feature = "chunk-borders")]
        application.init_resource::<chunk_borders::ChunkBordersState>();

        let plugins = (
            EguiPlugin,
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            WireframePlugin,
            WorldInspectorPlugin::new(),
            #[cfg(feature = "chunk-borders")]
            MaterialPlugin::<LineMaterial>::default(),
        );
        let systems = (
            display_debug_info.run_if(resource_equals(UiState {
                show_debug_info: true,
            })),
            toggle_debug_info,
            wireframe::toggle,
            #[cfg(feature = "chunk-borders")]
            chunk_borders::toggle,
        );
        application
            .add_plugins(plugins)
            .add_systems(Update, systems);
    }
}
