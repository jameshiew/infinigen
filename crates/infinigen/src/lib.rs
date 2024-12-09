//! Bevy-specific code and the entrypoint for the application.
use bevy::prelude::{AppExtStates, Plugin, States};

pub mod assets;
pub mod camera;
pub mod chunks;
pub mod cursor;
#[cfg(feature = "debug-ui")]
pub mod debug;
pub mod scene;
pub mod settings;
pub mod utils;
pub mod world;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadingAssets,
    InitializingRegistry,
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
    fn build(&self, app: &mut bevy::prelude::App) {
        tracing::info!("Initializing app plugin with config: {:#?}", self.config);
        app.init_state::<AppState>()
            .insert_resource(self.config.clone())
            .add_plugins((
                assets::Plugin,
                scene::Plugin,
                chunks::ChunksPlugin,
                camera::CameraPlugin,
                cursor::CursorPlugin,
                world::WorldPlugin,
                #[cfg(feature = "debug-ui")]
                debug::UiPlugin,
            ));
    }
}
