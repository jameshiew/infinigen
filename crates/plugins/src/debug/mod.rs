use bevy::camera::Camera3dDepthLoadOp;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::gizmos::config::GizmoConfigStore;
use bevy::light::cluster::ClusterConfig;
use bevy::light::{AmbientLight, DirectionalLight, PointLight};
use bevy::prelude::*;
use bevy::render::view::{ColorGradingGlobal, ColorGradingSection};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_inspector_egui::bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use leafwing_input_manager::prelude::*;

use self::info::{UiState, display_debug_info};
use self::world_inspector::world_inspector_ui;
use crate::AppState;

mod info;
#[cfg(not(target_family = "wasm"))]
mod wireframe;
mod world_inspector;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing debug UI plugin");
        app.register_type::<GizmoConfigStore>();
        app.register_type::<ColorGradingSection>();
        app.register_type::<ColorGradingGlobal>();
        app.register_type::<AmbientLight>();
        app.register_type::<PointLight>();
        app.register_type::<DirectionalLight>();
        app.register_type::<ClusterConfig>();
        app.register_type::<Camera3dDepthLoadOp>();
        app.init_resource::<UiState>()
            .add_plugins((
                InputManagerPlugin::<info::Action>::default(),
                EguiPlugin::default(),
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                DefaultInspectorConfigPlugin,
            ))
            .add_systems(OnEnter(AppState::MainGame), info::setup_actions)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    display_debug_info.run_if(resource_equals(UiState {
                        show_debug_info: true,
                    })),
                    info::handle_actions,
                    world_inspector_ui.run_if(resource_equals(UiState {
                        show_debug_info: true,
                    })),
                )
                    .run_if(in_state(AppState::MainGame)),
            );
        #[cfg(not(target_family = "wasm"))]
        app.add_plugins(wireframe::WireframePlugin);
    }
}
