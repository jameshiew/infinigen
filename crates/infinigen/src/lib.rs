//! Bevy-specific code and the entrypoint for the application.
use bevy::prelude::{AppExtStates, Plugin, States};
#[cfg(all(feature = "remote", not(target_family = "wasm")))]
use bevy::remote::http::RemoteHttpPlugin;
#[cfg(all(feature = "remote", not(target_family = "wasm")))]
use bevy::remote::RemotePlugin;

pub mod assets;
pub mod camera;
pub mod chunks;
pub mod debug;
#[cfg(all(
    feature = "jemalloc",
    not(target_env = "msvc"),
    not(target_family = "wasm")
))]
pub mod global_allocator;
pub mod mesh;
pub mod scene;
pub mod settings;
pub mod utils;
pub mod world;

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
    fn build(&self, app: &mut bevy::prelude::App) {
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
                #[cfg(all(feature = "remote", not(target_family = "wasm")))]
                RemotePlugin::default(),
                #[cfg(all(feature = "remote", not(target_family = "wasm")))]
                RemoteHttpPlugin::default(),
            ));
    }
}
