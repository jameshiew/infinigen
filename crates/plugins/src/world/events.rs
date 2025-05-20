use std::collections::hash_map::Entry;
use std::sync::Arc;

use ahash::AHashMap;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use infinigen_common::blocks::BlockVisibility;
use infinigen_common::chunks::{Array3Chunk, CHUNK_SIZE};
use infinigen_common::mesh::faces::{BlockVisibilityChecker, extract_faces};
use infinigen_common::world::{BlockPosition, ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use super::{ChunkInfo, ChunkStatus};
use crate::assets::blocks::BlockRegistry;

#[derive(Event)]
pub struct GenerateChunkRequest {
    pub chunk_position: ChunkPosition,
    pub zoom_level: ZoomLevel,
}

#[derive(Component)]
pub struct GenerateChunkTask(pub Task<(ZoomLevel, ChunkPosition, Option<ChunkInfo>)>);

pub fn handle_generate_chunk_request(
    mut commands: Commands,
    mut generate_chunk_reqs: EventReader<GenerateChunkRequest>,
    mut world: ResMut<crate::world::World>,
    registry: Res<BlockRegistry>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for (
        GenerateChunkRequest {
            chunk_position,
            zoom_level,
        },
        _,
    ) in generate_chunk_reqs.par_read()
    {
        match world.cache.entry((*chunk_position, *zoom_level)) {
            Entry::Occupied(_) => {
                tracing::trace!(
                    ?chunk_position,
                    ?zoom_level,
                    "Ignored request for previously requested chunk"
                );
                continue;
            }
            Entry::Vacant(enty) => {
                enty.insert(ChunkStatus::Generating);
            }
        };
        let task = generate_chunk_async(
            task_pool,
            *zoom_level,
            *chunk_position,
            world.generator.clone(),
            Box::new(registry.definitions.visibility_checker()),
        );
        commands.spawn((Name::new("Generate chunk task"), GenerateChunkTask(task)));
    }
}

pub fn generate_chunk_async(
    task_pool: &AsyncComputeTaskPool,
    zoom_level: ZoomLevel,
    position: ChunkPosition,
    worldgen: Arc<dyn WorldGen + Send + Sync>,
    visibility_checker: Box<impl BlockVisibilityChecker + 'static>,
) -> Task<(ZoomLevel, ChunkPosition, Option<ChunkInfo>)> {
    let zoom_level = zoom_level.to_owned();
    let position = position.to_owned();
    task_pool.spawn(async move {
        let chunk = worldgen.get(&position, zoom_level);
        let Some(mut chunk) = chunk else {
            return (zoom_level, position, None);
        };
        let faces = extract_faces(&chunk, *visibility_checker.clone());
        let translucents = split_out_translucent(&mut chunk, *visibility_checker);
        (
            zoom_level,
            position,
            Some(ChunkInfo {
                opaque: Box::new(chunk),
                faces,
                translucents: translucents.into_values().collect(),
            }),
        )
    })
}

pub fn handle_generate_chunk_task(
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
    mut generate_chunk_tasks: Query<(Entity, &mut GenerateChunkTask)>,
) {
    for (entity, mut task) in generate_chunk_tasks.iter_mut() {
        if let Some((zoom_level, cpos, chunk)) = block_on(poll_once(&mut task.0)) {
            let status = chunk.map_or_else(
                || ChunkStatus::Empty,
                |chunk_info| ChunkStatus::Generated(Arc::new(chunk_info)),
            );
            match world.cache.insert((cpos, zoom_level), status) {
                Some(status) => match status {
                    ChunkStatus::Generating => (),
                    ChunkStatus::Generated(_) | ChunkStatus::Empty => {
                        tracing::debug!(?cpos, ?zoom_level, "Overwrote already generated chunk!")
                    }
                },
                None => unreachable!(),
            };
            commands.entity(entity).despawn();
        }
    }
}

/// Splits out translucent chunks from chunk, leaving only opaque blocks.
pub fn split_out_translucent(
    chunk: &mut Array3Chunk,
    visibility_checker: impl BlockVisibilityChecker,
) -> AHashMap<MappedBlockID, Array3Chunk> {
    let mut translucents = AHashMap::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if let Some(block_id) = chunk.get(&BlockPosition { x, y, z }) {
                    if visibility_checker.get_visibility(&block_id) == BlockVisibility::Opaque {
                        continue;
                    }
                    translucents
                        .entry(block_id)
                        .or_insert_with(Array3Chunk::default)
                        .insert(&BlockPosition { x, y, z }, block_id);
                    chunk.clear(&BlockPosition { x, y, z });
                }
            }
        }
    }

    translucents
}
