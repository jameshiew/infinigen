use std::sync::Arc;

use ahash::AHashMap;
use bevy::prelude::*;
use events::{GenerateChunkRequest, GenerateChunkTask};
use infinigen_common::blocks::Palette;
use infinigen_common::chunks::{Array3Chunk, Chunk};
use infinigen_common::mesh::shapes::ChunkFace;
use infinigen_common::world::{ChunkPosition, Direction, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use linearize::StaticCopyMap;

use crate::assets::blocks::BlockRegistry;
use crate::AppState;

pub mod events;

#[derive(Resource)]
pub struct World {
    pub generator: Arc<dyn WorldGen + Send + Sync>,
    pub cache: AHashMap<(ChunkPosition, ZoomLevel), ChunkStatus>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub opaque: Box<Array3Chunk>,
    pub translucents: Vec<Array3Chunk>,
    pub faces: StaticCopyMap<Direction, ChunkFace>,
}

pub enum ChunkStatus {
    Generating,
    Generated(Arc<ChunkInfo>),
    Empty,
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
            cache: Default::default(),
        }
    }
}

impl World {
    pub fn generate(&self, zoom_level: ZoomLevel, pos: &ChunkPosition) -> Chunk {
        self.generator.get(pos, zoom_level)
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing world plugin");
        app.init_resource::<World>()
            .add_event::<GenerateChunkRequest>()
            .add_systems(OnEnter(AppState::InitializingWorld), setup)
            .add_systems(
                Update,
                (
                    events::handle_generate_chunk_request.run_if(on_event::<GenerateChunkRequest>),
                    events::handle_generate_chunk_task
                        .run_if(any_with_component::<GenerateChunkTask>),
                )
                    .run_if(in_state(AppState::MainGame)),
            );
    }
}

#[derive(Resource)]
pub struct WorldSettings {
    pub world: Box<dyn Fn(Palette) -> Arc<dyn WorldGen + Send + Sync> + Send + Sync>,
}

fn setup(
    mut next_state: ResMut<NextState<AppState>>,
    registry: Res<BlockRegistry>,
    settings: Res<WorldSettings>,
    mut world: ResMut<World>,
) {
    world.generator = (*settings.world)(registry.definitions.palette());
    next_state.set(AppState::MainGame);
}
