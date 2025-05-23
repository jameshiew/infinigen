use bevy::prelude::*;

pub mod assets;
pub mod camera;
pub mod controls;
pub mod debug;
pub mod mesh;
pub mod scene;
pub mod world;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadingAssets,
    InitializingRegistry,
    InitializingWorld,
    MainGame,
    Paused,
}
