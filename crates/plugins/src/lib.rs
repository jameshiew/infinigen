use bevy::prelude::*;

pub mod assets;
pub mod camera;
pub mod debug;
pub mod mesh;
pub mod registry;
pub mod scene;
pub mod settings;
pub mod window;
pub mod world;

use crate::camera::setup::CameraSettings;
use crate::scene::SceneSettings;
use crate::world::WorldSettings;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadingAssets,
    InitializingRegistry,
    InitializingWorld,
    MainGame,
    Paused,
}

pub struct AppPlugin {
    settings: settings::AppSettings,
}

impl AppPlugin {
    pub const fn new(settings: settings::AppSettings) -> Self {
        Self { settings }
    }
}

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing app plugin with config: {:#?}", self.settings);
        app.init_state::<AppState>()
            .insert_resource(CameraSettings {
                zoom_level: self.settings.zoom_level,
                target_x: self.settings.target_x as f32,
                target_y: self.settings.target_y as f32,
                target_z: self.settings.target_z as f32,
                wx: self.settings.wx as f32,
                wy: self.settings.wy as f32,
                wz: self.settings.wz as f32,
            })
            .insert_resource(SceneSettings {
                horizontal_view_distance: self.settings.horizontal_view_distance as usize,
                vertical_view_distance: self.settings.vertical_view_distance as usize,
                zoom_level: self.settings.zoom_level,
            })
            .insert_resource(WorldSettings {
                world_gen_name: self.settings.world.clone(),
                seed: self.settings.seed as u32,
            })
            .add_plugins((
                registry::RegistryPlugin,
                assets::AssetsPlugin,
                scene::ScenePlugin,
                mesh::MeshPlugin,
                camera::CameraPlugin,
                world::WorldPlugin,
                debug::DebugPlugin,
                window::ControlsPlugin,
            ));
    }
}
