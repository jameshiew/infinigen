use bevy::{pbr::wireframe::WireframeConfig, prelude::*};

pub fn toggle(keys: Res<ButtonInput<KeyCode>>, mut wireframe_cfg: ResMut<WireframeConfig>) {
    for key in keys.get_just_pressed() {
        if key == &KeyCode::F3 {
            wireframe_cfg.global = !wireframe_cfg.global;
            tracing::info!(%wireframe_cfg.global, "Wireframe toggled");
        }
    }
}
