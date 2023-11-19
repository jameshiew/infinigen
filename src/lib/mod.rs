#![feature(test)]

use bevy::prelude::Msaa;
use bevy::prelude::Plugin;

pub mod camera;
pub mod chunks;
pub mod common;
pub mod cursor;
pub mod debug;
pub mod extras;
pub mod fake_client;
pub mod mesh;
pub mod render;
pub mod scene;
pub mod settings;

pub struct ClientPlugin {
    config: settings::Config,
}

impl ClientPlugin {
    pub fn new(config: settings::Config) -> Self {
        Self { config }
    }
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        tracing::info!("Initializing client plugin with config: {:#?}", self.config);
        app.insert_resource(self.config.clone())
            .insert_resource(Msaa::Off)
            .add_plugins((
                scene::Plugin,
                chunks::ChunksPlugin,
                camera::CameraPlugin,
                cursor::CursorPlugin,
                fake_client::FakeClientPlugin,
                debug::UiPlugin,
            ));
    }
}
