use bevy::prelude::*;
use infinigen_plugins::camera::setup::CameraSettings;
use infinigen_plugins::scene::{self, SceneSettings};
use infinigen_plugins::world::{self, WorldSettings};
use infinigen_plugins::{assets, camera, debug, mesh, AppState};

pub mod settings;

pub struct AppPlugin {
    settings: settings::AppSettings,
}

impl AppPlugin {
    pub fn new(settings: settings::AppSettings) -> Self {
        Self { settings }
    }
}

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing app plugin with config: {:#?}", self.settings);
        app.init_state::<AppState>()
            .insert_resource(CameraSettings {
                zoom_level: self.settings.zoom_level,
                rotation_x: self.settings.rotation_x as f32,
                rotation_y: self.settings.rotation_y as f32,
                rotation_z: self.settings.rotation_z as f32,
                rotation_w: self.settings.rotation_w as f32,
                wx: self.settings.wx as f32,
                wy: self.settings.wy as f32,
                wz: self.settings.wz as f32,
            })
            .insert_resource(SceneSettings {
                hview_distance: self.settings.hview_distance as usize,
                vview_distance: self.settings.vview_distance as usize,
                zoom_level: self.settings.zoom_level,
            })
            .insert_resource(WorldSettings {
                world_gen_name: self.settings.world.clone(),
                seed: self.settings.seed as u32,
            })
            .add_plugins((
                assets::AssetsPlugin,
                scene::ScenePlugin,
                mesh::MeshPlugin,
                camera::CameraPlugin,
                world::WorldPlugin,
                debug::DebugPlugin,
            ));
    }
}
