use bevy::prelude::*;
use registry::ChunkRegistry;
use tasks::RequestChunkEvent;

use crate::AppState;

pub mod registry;
pub mod tasks;

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing chunks plugin");
        app.init_resource::<ChunkRegistry>()
            .add_event::<RequestChunkEvent>()
            .add_systems(
                Update,
                (
                    tasks::handle_chunk_request.run_if(on_event::<RequestChunkEvent>),
                    tasks::handle_chunk_finished_generating,
                )
                    .run_if(in_state(AppState::MainGame)),
            );
    }
}
