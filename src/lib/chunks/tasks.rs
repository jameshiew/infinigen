use bevy::{prelude::*, tasks::Task};
use futures_lite::future;

use crate::{
    common::{chunks::Chunk, world::ChunkPosition},
    scene::assets,
};

use super::registry::ChunkRegistry;

#[derive(Component)]
pub struct GenerateChunk(pub Task<(i8, ChunkPosition, Chunk)>);

pub fn handle(
    mut commands: Commands,
    assets_registry: Res<assets::Registry>,
    mut registry: ResMut<ChunkRegistry>,
    mut transform_tasks: Query<(Entity, &mut GenerateChunk)>,
) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some((zoom_level, cpos, chunk)) = future::block_on(future::poll_once(&mut task.0)) {
            registry.insert(&zoom_level, &cpos, chunk, &assets_registry.block_mappings);
            commands.entity(entity).despawn();
        }
    }
}
