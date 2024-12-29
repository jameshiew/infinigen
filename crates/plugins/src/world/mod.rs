use std::str::FromStr;
use std::sync::Arc;

use bevy::prelude::*;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use infinigen_extras::worldgen::{self, WorldGenTypes};

use crate::assets::blocks::BlockRegistry;
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

#[derive(Resource)]
pub struct WorldSettings {
    pub world: String,
}

fn init_world(
    mut next_state: ResMut<NextState<AppState>>,
    registry: Res<BlockRegistry>,
    settings: Res<WorldSettings>,
    mut world: ResMut<World>,
) {
    let world_gen_type = WorldGenTypes::from_str(&settings.world)
        .unwrap_or_else(|_| panic!("couldn't parse world gen type from {}", &settings.world));
    world.generator = world_gen_type.as_world_gen(registry.definitions.palette());
    next_state.set(AppState::MainGame);
}
