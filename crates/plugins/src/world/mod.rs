use std::sync::Arc;

use bevy::prelude::*;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use infinigen_extras::worldgen;

use crate::assets::blocks::BlockRegistry;
use crate::settings::Config;
use crate::AppState;

#[derive(Resource)]
pub struct World {
    pub generator: Arc<dyn WorldGen + Send + Sync>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            generator: Arc::new(worldgen::flat::Flat::default()),
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
        app.init_resource::<World>()
            .add_systems(OnEnter(AppState::InitializingWorld), init_world);
    }
}

fn init_world(
    mut next_state: ResMut<NextState<AppState>>,
    registry: Res<BlockRegistry>,
    config: Res<Config>,
    mut world: ResMut<World>,
) {
    world.generator = config.world.as_world_gen(registry.block_mappings.palette());
    next_state.set(AppState::MainGame);
}
