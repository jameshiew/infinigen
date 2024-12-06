use bevy::prelude::*;
use registry::ChunkRegistry;
use tasks::RequestChunkEvent;

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
                    tasks::handle_chunk_request,
                    tasks::handle_chunk_finished_generating,
                ),
            );
    }
}
