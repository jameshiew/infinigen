use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::AppState;

use self::chunk_borders::LineMaterial;
use self::info::{display_debug_info, toggle_debug_info, UiState};

mod chunk_borders;
mod info;
mod wireframe;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing debug UI plugin");
        app.init_resource::<UiState>()
            .init_resource::<chunk_borders::ChunkBordersState>()
            .add_plugins((
                EguiPlugin,
                FrameTimeDiagnosticsPlugin,
                EntityCountDiagnosticsPlugin,
                WireframePlugin,
                WorldInspectorPlugin::new().run_if(resource_equals(UiState {
                    show_debug_info: true,
                })),
                MaterialPlugin::<LineMaterial>::default(),
            ))
            .add_systems(
                Update,
                (
                    display_debug_info.run_if(resource_equals(UiState {
                        show_debug_info: true,
                    })),
                    toggle_debug_info,
                    wireframe::toggle,
                    chunk_borders::toggle,
                )
                    .run_if(in_state(AppState::MainGame)),
            );
    }
}
