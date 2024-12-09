use std::sync::Arc;

use bevy::core::Name;
use bevy::prelude::{Commands, Component, Entity, Event, EventReader, Query, Res, ResMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use super::registry::{get_neighbour_cposes, ChunkRegistry, ChunkStatus};
use crate::assets;
use crate::world::World;

#[derive(Component)]
pub struct GenerateChunkTask(pub Task<(ZoomLevel, ChunkPosition, Chunk)>);

#[derive(Event)]
pub struct RequestChunkEvent {
    pub zoom_level: ZoomLevel,
    pub position: ChunkPosition,
}

pub fn handle_chunk_request(
    mut commands: Commands,
    mut request_chunk_evs: EventReader<RequestChunkEvent>,
    mut registry: ResMut<ChunkRegistry>,
    world: Res<World>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for (ev, _) in request_chunk_evs.par_read() {
        if registry.get_status(ev.zoom_level, &ev.position).is_some() {
            continue;
        }
        registry.set_status(ev.zoom_level, &ev.position, ChunkStatus::Requested);
        let task = generate_chunk_async(
            task_pool,
            ev.zoom_level,
            ev.position,
            world.generator.clone(),
        );
        commands.spawn((Name::new("Generate chunk task"), task));
        // request neighbours directly also, so that the above chunk can be meshed later
        for (_, neighbour_cpos) in get_neighbour_cposes(&ev.position) {
            if registry
                .get_status(ev.zoom_level, &neighbour_cpos)
                .is_some()
            {
                continue;
            }
            registry.set_status(ev.zoom_level, &neighbour_cpos, ChunkStatus::Requested);
            let task = generate_chunk_async(
                task_pool,
                ev.zoom_level,
                neighbour_cpos,
                world.generator.clone(),
            );
            commands.spawn((Name::new("Generate chunk task"), task));
        }
    }
}

pub fn generate_chunk_async(
    task_pool: &AsyncComputeTaskPool,
    zoom_level: ZoomLevel,
    position: ChunkPosition,
    worldgen: Arc<Box<dyn WorldGen + Send + Sync>>,
) -> GenerateChunkTask {
    let zoom_level = zoom_level.to_owned();
    let position = position.to_owned();
    let task =
        task_pool.spawn(async move { (zoom_level, position, worldgen.get(&position, zoom_level)) });
    GenerateChunkTask(task)
}

pub fn handle_chunk_finished_generating(
    mut commands: Commands,
    assets_registry: Res<assets::blocks::BlockRegistry>,
    mut registry: ResMut<ChunkRegistry>,
    mut transform_tasks: Query<(Entity, &mut GenerateChunkTask)>,
) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some((zoom_level, cpos, chunk)) = future::block_on(future::poll_once(&mut task.0)) {
            registry.insert_generated(zoom_level, &cpos, chunk, &assets_registry.block_mappings);
            commands.entity(entity).despawn();
        }
    }
}
