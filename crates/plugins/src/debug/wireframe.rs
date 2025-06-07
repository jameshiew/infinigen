use bevy::pbr::wireframe::WireframeConfig;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::AppState;

pub struct WireframePlugin;

impl Plugin for WireframePlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing wireframe plugin");

        app.add_plugins((
            bevy::pbr::wireframe::WireframePlugin::default(),
            InputManagerPlugin::<Action>::default(),
        ))
        .add_systems(OnEnter(AppState::MainGame), setup_actions)
        .add_systems(Update, handle_actions.run_if(in_state(AppState::MainGame)));
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    ToggleWireframes,
}

pub fn setup_actions(mut commands: Commands) {
    let input_map = InputMap::new([(Action::ToggleWireframes, KeyCode::F3)]);
    commands.spawn(input_map);
}

pub fn handle_actions(
    action_state: Single<&ActionState<Action>>,
    mut wireframe_cfg: ResMut<WireframeConfig>,
) {
    if action_state.just_pressed(&Action::ToggleWireframes) {
        wireframe_cfg.global = !wireframe_cfg.global;
        tracing::info!(%wireframe_cfg.global, "Wireframe toggled");
    }
}
