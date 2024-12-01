use bevy::core::Name;
use bevy::prelude::{Commands, Component, Entity, Event, EventReader, Query, Res, ResMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

use crate::assets;
use crate::world::World;
use infinigen_common::zoom::ZoomLevel;
use infinigen_common::{chunks::Chunk, world::ChunkPosition};

use super::registry::{get_neighbour_cposes, ChunkRegistry, ChunkStatus};

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
    let thread_pool = AsyncComputeTaskPool::get();
    for ev in request_chunk_evs.read() {
        if registry.get_status(ev.zoom_level, &ev.position).is_some() {
            continue;
        }
        registry.set_status(ev.zoom_level, &ev.position, ChunkStatus::Requested);
        let worldgen = world.generator.clone();
        let zoom_level = ev.zoom_level.to_owned();
        let position = ev.position.to_owned();
        let task = thread_pool
            .spawn(async move { (zoom_level, position, worldgen.get(&position, zoom_level)) });
        commands.spawn((Name::new("Generate chunk task"), GenerateChunkTask(task)));
        // request neighbours directly also, so that the above chunk can be meshed later
        for (_, neighbour_cpos) in get_neighbour_cposes(&ev.position).into_iter() {
            if registry
                .get_status(ev.zoom_level, &neighbour_cpos)
                .is_some()
            {
                continue;
            }
            let worldgen = world.generator.clone();
            let task = thread_pool.spawn(async move {
                (
                    zoom_level,
                    neighbour_cpos,
                    worldgen.get(&neighbour_cpos, zoom_level),
                )
            });
            commands.spawn((Name::new("Generate chunk task"), GenerateChunkTask(task)));
        }
    }
}

pub fn handle_chunk_finished_generating(
    mut commands: Commands,
    assets_registry: Res<assets::Registry>,
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
