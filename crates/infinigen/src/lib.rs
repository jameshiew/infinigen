use bevy::prelude::Plugin;

pub mod camera;
pub mod chunks;
pub mod cursor;
pub mod debug;
pub mod mesh;
pub mod render;
pub mod scene;
pub mod settings;
pub mod world;

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
        app.insert_resource(self.config.clone()).add_plugins((
            scene::Plugin,
            chunks::ChunksPlugin,
            camera::CameraPlugin,
            cursor::CursorPlugin,
            world::WorldPlugin,
            debug::UiPlugin,
        ));
    }
}
