use bevy::prelude::*;

use registry::ChunkRegistry;

pub mod registry;
pub mod tasks;

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing chunks plugin");
        app.init_resource::<ChunkRegistry>()
            .add_systems(Update, (tasks::handle,));
    }
}
