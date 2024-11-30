use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use crate::extras::worldgen;

#[derive(Resource)]
pub struct FakeClient {
    pub world: Arc<RwLock<Box<dyn WorldGen + Send + Sync>>>,
}

impl Default for FakeClient {
    fn default() -> Self {
        let x: Box<dyn WorldGen + Send + Sync> = Box::<worldgen::flat::Flat>::default();
        Self {
            world: Arc::new(RwLock::new(x)),
        }
    }
}

/// Eventually will become a trait.
impl FakeClient {
    pub fn get_chunk(&self, zoom_level: ZoomLevel, pos: &ChunkPosition) -> Chunk {
        self.world.write().unwrap().get(pos, zoom_level)
    }
}

pub struct FakeClientPlugin;

impl Plugin for FakeClientPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing fake client plugin");
        app.init_resource::<FakeClient>();
    }
}
