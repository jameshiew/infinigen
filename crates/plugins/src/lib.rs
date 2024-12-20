use bevy::prelude::States;

pub mod assets;
pub mod camera;
pub mod chunks;
pub mod debug;
pub mod scene;
pub mod settings;
pub mod world;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadingAssets,
    InitializingRegistry,
    InitializingWorld,
    MainGame,
}
