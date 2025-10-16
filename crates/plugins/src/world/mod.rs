use std::sync::Arc;

use ahash::AHashMap;
use anyhow::Context;
use bevy::prelude::*;
use infinigen_common::blocks::Palette;
use infinigen_common::chunks::Array3Chunk;
use infinigen_common::mesh::shapes::ChunkFace;
use infinigen_common::world::{ChunkPosition, Direction, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use linearize::StaticCopyMap;
use messages::{GenerateChunkRequest, GenerateChunkTask};

use crate::AppState;
use crate::registry::BlockRegistry;

pub mod messages;

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
            fn get(&self, _pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Option<Array3Chunk> {
                None
            }
        }
        Self {
            generator: Arc::new(Empty),
            cache: Default::default(),
        }
    }
}

impl World {
    pub fn generate(&self, zoom_level: ZoomLevel, pos: &ChunkPosition) -> Option<Array3Chunk> {
        self.generator.get(pos, zoom_level)
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing world plugin");
        app.init_resource::<World>()
            .add_message::<GenerateChunkRequest>()
            .add_systems(OnEnter(AppState::InitializingWorld), init_world)
            .add_systems(
                FixedUpdate,
                (
                    messages::handle_generate_chunk_request
                        .run_if(on_message::<GenerateChunkRequest>),
                    messages::handle_generate_chunk_task
                        .run_if(any_with_component::<GenerateChunkTask>),
                )
                    .run_if(in_state(AppState::MainGame)),
            );
    }
}

pub type WorldInitializerFn = Box<
    dyn Fn(&str, u32, Palette) -> anyhow::Result<Arc<dyn WorldGen + Send + Sync>> + Send + Sync,
>;

#[derive(Resource)]
pub struct WorldInitializer(pub WorldInitializerFn);

#[derive(Resource)]
pub struct WorldSettings {
    pub world_gen_name: String,
    pub seed: u32,
}

fn init_world(
    mut next_state: ResMut<NextState<AppState>>,
    registry: Res<BlockRegistry>,
    world_initializer: Res<WorldInitializer>,
    settings: Res<WorldSettings>,
    mut world: ResMut<World>,
) -> Result {
    let WorldInitializer(world_initializer) = &*world_initializer;
    let world_gen_name = &settings.world_gen_name;
    world.generator = world_initializer(
        world_gen_name,
        settings.seed,
        registry.definitions.palette(),
    )
    .with_context(|| format!("No world generator found for '{world_gen_name}'"))?;
    next_state.set(AppState::MainGame);
    Ok(())
}
