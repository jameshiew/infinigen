use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::prelude::*;

use self::info::{UiState, display_debug_info};
use crate::AppState;

mod info;
#[cfg(not(target_family = "wasm"))]
mod wireframe;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing debug UI plugin");
        app.init_resource::<UiState>()
            .add_plugins((
                InputManagerPlugin::<info::Action>::default(),
                EguiPlugin {
                    enable_multipass_for_primary_context: false,
                },
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin,
                WorldInspectorPlugin::default().run_if(resource_equals(UiState {
                    show_debug_info: true,
                })),
            ))
            .add_systems(OnEnter(AppState::MainGame), info::setup_actions)
            .add_systems(
                Update,
                (
                    display_debug_info.run_if(resource_equals(UiState {
                        show_debug_info: true,
                    })),
                    info::handle_actions,
                )
                    .run_if(in_state(AppState::MainGame)),
            );
        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::pbr::wireframe::WireframePlugin;

            app.add_plugins((
                WireframePlugin::default(),
                InputManagerPlugin::<wireframe::Action>::default(),
            ))
            .add_systems(OnEnter(AppState::MainGame), wireframe::setup_actions)
            .add_systems(
                Update,
                (wireframe::handle_actions).run_if(in_state(AppState::MainGame)),
            );
        }
    }
}
