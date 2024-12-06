use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use self::info::{display_debug_info, toggle_debug_info, UiState};
use crate::AppState;

#[cfg(not(target_family = "wasm"))]
mod chunk_borders;
mod info;
#[cfg(not(target_family = "wasm"))]
mod wireframe;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing debug UI plugin");
        app.init_resource::<UiState>()
            .add_plugins((
                EguiPlugin,
                FrameTimeDiagnosticsPlugin,
                EntityCountDiagnosticsPlugin,
                WorldInspectorPlugin::new().run_if(resource_equals(UiState {
                    show_debug_info: true,
                })),
            ))
            .add_systems(
                Update,
                (
                    display_debug_info.run_if(resource_equals(UiState {
                        show_debug_info: true,
                    })),
                    toggle_debug_info,
                )
                    .run_if(in_state(AppState::MainGame)),
            );
        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::pbr::wireframe::WireframePlugin;

            app.init_resource::<chunk_borders::ChunkBordersState>()
                .add_plugins((
                    WireframePlugin,
                    MaterialPlugin::<chunk_borders::LineMaterial>::default(),
                ))
                .add_systems(
                    Update,
                    (wireframe::toggle, chunk_borders::toggle).run_if(in_state(AppState::MainGame)),
                );
        }
    }
}
