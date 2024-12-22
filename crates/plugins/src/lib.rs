use bevy::prelude::*;

mod assets;
mod camera;
mod chunks;
mod debug;
mod scene;
pub mod settings;
mod world;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadingAssets,
    InitializingRegistry,
    InitializingWorld,
    MainGame,
}

pub struct AppPlugin {
    config: settings::Config,
}

impl AppPlugin {
    pub fn new(config: settings::Config) -> Self {
        Self { config }
    }
}

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing app plugin with config: {:#?}", self.config);
        app.init_state::<AppState>()
            .insert_resource(self.config.clone())
            .add_plugins((
                assets::AssetsPlugin,
                scene::ScenePlugin,
                chunks::ChunksPlugin,
                camera::CameraPlugin,
                world::WorldPlugin,
                debug::DebugPlugin,
            ));
    }
}
