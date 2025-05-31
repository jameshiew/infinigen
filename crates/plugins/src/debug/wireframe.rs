use bevy::pbr::wireframe::WireframeConfig;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

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
