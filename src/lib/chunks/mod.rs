use bevy::prelude::*;

use registry::ChunkRegistry;

pub mod registry;
pub mod tasks;

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkRegistry>()
            .add_system(tasks::handle);
    }
}
