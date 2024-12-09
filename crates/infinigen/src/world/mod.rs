use std::sync::Arc;

use bevy::prelude::*;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use infinigen_extras::worldgen;

#[derive(Resource)]
pub struct World {
    pub generator: Arc<Box<dyn WorldGen + Send + Sync>>,
}

impl Default for World {
    fn default() -> Self {
        let x: Box<dyn WorldGen + Send + Sync> = Box::<worldgen::flat::Flat>::default();
        Self {
            generator: Arc::new(x),
        }
    }
}

impl World {
    pub fn get_chunk(&self, zoom_level: ZoomLevel, pos: &ChunkPosition) -> Chunk {
        self.generator.get(pos, zoom_level)
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing world plugin");
        app.init_resource::<World>();
    }
}
