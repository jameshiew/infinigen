use std::sync::{Arc, RwLock};

use crate::common::chunks::Chunk;
use crate::common::world::{ChunkPosition, WorldGen};
use crate::extras::worldgen;
use bevy::prelude::*;

#[derive(Resource)]
pub struct FakeClient {
    pub world: Arc<RwLock<Box<dyn WorldGen + Send + Sync>>>,
}

#[allow(clippy::derivable_impls)] // https://github.com/rust-lang/rust-clippy/issues/10158
impl Default for FakeClient {
    fn default() -> Self {
        let x: Box<dyn WorldGen + Send + Sync> =
            Box::<worldgen::Flat>::default();
        Self {
            world: Arc::new(RwLock::new(x)),
        }
    }
}

/// Eventually will become a trait.
impl FakeClient {
    pub fn get_chunk(&self, zoom_level: i8, pos: &ChunkPosition) -> Chunk {
        self.world
            .read()
            .unwrap()
            .get(pos, 2f64.powf(zoom_level as f64))
    }
}

pub struct FakeClientPlugin;

impl Plugin for FakeClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FakeClient>();
    }
}
