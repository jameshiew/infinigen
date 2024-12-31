use std::sync::Arc;

use bevy::prelude::*;
use infinigen_common::blocks::Palette;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::assets::blocks::BlockRegistry;
use crate::AppState;

#[derive(Resource)]
pub struct World {
    pub generator: Arc<dyn WorldGen + Send + Sync>,
}

impl Default for World {
    fn default() -> Self {
        struct Empty;
        impl WorldGen for Empty {
            fn get(&self, _pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
                Chunk::Empty
            }
        }
        Self {
            generator: Arc::new(Empty),
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
    pub world: Box<dyn Fn(Palette) -> Arc<dyn WorldGen + Send + Sync> + Send + Sync>,
}

fn init_world(
    mut next_state: ResMut<NextState<AppState>>,
    registry: Res<BlockRegistry>,
    settings: Res<WorldSettings>,
    mut world: ResMut<World>,
) {
    world.generator = (*settings.world)(registry.definitions.palette());
    next_state.set(AppState::MainGame);
}
