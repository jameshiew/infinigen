use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use self::info::{display_debug_info, toggle_debug_info, UiState};
use crate::AppState;

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
                WorldInspectorPlugin::default().run_if(resource_equals(UiState {
                    show_debug_info: true,
                })),
            ))
            .add_systems(
                Update,
                (
                    display_debug_info.run_if(resource_equals(UiState {
                        show_debug_info: true,
                    })),
                    toggle_debug_info.run_if(resource_changed::<ButtonInput<KeyCode>>),
                )
                    .run_if(in_state(AppState::MainGame)),
            );
        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::pbr::wireframe::WireframePlugin;

            app.add_plugins((WireframePlugin,)).add_systems(
                Update,
                wireframe::toggle.run_if(
                    in_state(AppState::MainGame).and(resource_changed::<ButtonInput<KeyCode>>),
                ),
            );
        }
    }
}
