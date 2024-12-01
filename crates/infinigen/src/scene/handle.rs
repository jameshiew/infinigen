use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use rustc_hash::FxHashSet;

use crate::chunks::{registry::ChunkStatus, tasks::GenerateChunk};
use crate::scene::LoadChunkOp;
use crate::{assets::MaterialType, chunks::registry::ChunkRegistry};
use crate::{
    render::mesh::{bevy_mesh_greedy_quads, bevy_mesh_visible_block_faces},
    world::World,
};
use infinigen_common::{
    chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F32},
    world::{BlockPosition, BlockVisibility, ChunkBlockId, WorldPosition},
};

use crate::assets::Registry;

use super::UnloadChunkOpEvent;

// bigger chunks means go slower to prevent lag/stutter
const CHUNK_OP_RATE: usize = (16. * (32. / CHUNK_SIZE_F32)) as usize;

/// Split out blocks from this chunk.
pub fn split(mut chunk: UnpackedChunk, chunk_block_id: ChunkBlockId) -> (UnpackedChunk, Chunk) {
    let mut split_out = UnpackedChunk::default();
    let mut contained_blocks_to_be_split_out = false;

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if let Some(block) = chunk.get(&BlockPosition { x, y, z }) {
                    if block == chunk_block_id {
                        contained_blocks_to_be_split_out = true;
                        split_out.insert(&BlockPosition { x, y, z }, block);
                        chunk.clear(&BlockPosition { x, y, z });
                    }
                }
            }
        }
    }

    let split_out = if contained_blocks_to_be_split_out {
        Chunk::Unpacked(Box::new(split_out))
    } else {
        Chunk::Empty
    };
    (chunk, split_out)
}

pub fn process_unload_chunk_ops(
    mut commands: Commands,
    mut unload_evs: EventReader<crate::scene::UnloadChunkOpEvent>,
    mut scene: ResMut<crate::scene::Scene>,
) {
    for _ in 0..CHUNK_OP_RATE {
        let Some(op) = unload_evs.read().next() else {
            return;
        };
        let UnloadChunkOpEvent(cpos) = op;

        if let Some(eids) = scene.loaded.get(cpos) {
            tracing::debug!(?cpos, "Unloading chunk");
            eids.iter().for_each(|physical_eid| {
                commands.entity(*physical_eid).despawn();
            });
            tracing::debug!(?cpos, "Chunk unloaded");
            scene.loaded.remove(cpos);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_load_chunk_ops(
    mut commands: Commands,
    mut chunks: ResMut<ChunkRegistry>,
    world: Res<World>,
    mut scene: ResMut<crate::scene::Scene>,
    mut load_ops: ResMut<crate::scene::LoadOps>,
    scene_zoom: Res<crate::scene::SceneZoom>,
    mut meshes: ResMut<Assets<Mesh>>,
    registry: Res<Registry>,
) {
    let scene_zoom_level = scene_zoom.zoom_level.into();
    let mut queued_generations = vec![];
    for _ in 0..CHUNK_OP_RATE {
        // loading
        let Some(op) = load_ops.deque.pop_front() else {
            return;
        };
        let LoadChunkOp(cpos) = op;

        match chunks.get_status(scene_zoom_level, &cpos) {
            Some(status) => match status {
                ChunkStatus::Present(_) => {
                    tracing::debug!(?cpos, "Will render chunk");
                }
                ChunkStatus::Generating => {
                    queued_generations.push(LoadChunkOp(cpos));
                    continue;
                }
            },
            None => {
                chunks.set_status(scene_zoom_level, &cpos, ChunkStatus::Generating);
                let thread_pool = AsyncComputeTaskPool::get();
                let worldgen = world.generator.clone();
                let task = thread_pool.spawn(async move {
                    (
                        scene_zoom_level,
                        cpos,
                        worldgen.write().unwrap().get(&cpos, scene_zoom_level),
                    )
                });
                commands.spawn((Name::new("Generate chunk task"), GenerateChunk(task)));
                queued_generations.push(LoadChunkOp(cpos));
                continue;
            }
        }

        let Chunk::Unpacked(chunk) = chunks.get_mut(
            scene_zoom_level,
            &cpos,
            world.as_ref(),
            &registry.block_mappings,
        ) else {
            tracing::debug!(?cpos, "Empty chunk");
            continue;
        };
        let opaque_mat = registry.get_material(MaterialType::DenseOpaque);
        let translucent_mat = registry.get_material(MaterialType::Translucent);
        let neighbor_faces = chunks.get_neighboring_faces_mut(
            scene_zoom_level,
            &cpos,
            world.as_ref(),
            &registry.block_mappings,
        );

        let mut loads = Vec::with_capacity(1); // most common case - only one mesh needed, for opaque blocks in chunk

        let mut opaque_chunk = *chunk;
        let block_textures = registry.get_block_textures();

        let translucent_chunk_block_ids: Vec<_> = registry
            .block_mappings
            .by_mapped_id
            .iter()
            .filter(|(_, block_definition)| {
                block_definition.visibility == BlockVisibility::Translucent
            })
            .map(|(chunk_block_id, _)| *chunk_block_id)
            .collect();

        for translucent_chunk_block_id in translucent_chunk_block_ids {
            let (remaining, translucent_chunk) = split(opaque_chunk, translucent_chunk_block_id);
            opaque_chunk = remaining;

            if let Chunk::Unpacked(translucent_chunk) = translucent_chunk {
                if let Some(translucent_mesh) = bevy_mesh_greedy_quads(
                    &translucent_chunk,
                    &neighbor_faces,
                    block_textures,
                    &registry.block_mappings,
                ) {
                    loads.push((translucent_mesh, translucent_mat.clone_weak()));
                }
            };
        }

        if let Some(opaque_mesh) = bevy_mesh_visible_block_faces(
            &opaque_chunk,
            &neighbor_faces,
            block_textures,
            &registry.block_mappings,
        ) {
            loads.push((opaque_mesh, opaque_mat));
        };

        if loads.is_empty() {
            tracing::debug!(?cpos, "Occluded chunk");
            continue;
        }

        let wpos: WorldPosition = (&cpos).into();
        let transform = Transform::from_xyz(wpos.x, wpos.y, wpos.z);

        // TODO: the above meshing stuff should be async also
        let mut eids = FxHashSet::default();
        for (mesh, material) in loads {
            let eid = commands
                .spawn((
                    Name::new("Chunk mesh"),
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material),
                    transform,
                ))
                .id();

            eids.insert(eid);
        }

        tracing::debug!(?cpos, ?eids, "Chunk loaded");
        scene.loaded.insert(cpos, eids);
    }
    for queued_generation in queued_generations.into_iter() {
        // prioritize rendering chunks we queued to generate this run
        load_ops.deque.push_front(queued_generation);
    }
}
