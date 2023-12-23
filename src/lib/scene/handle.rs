use std::collections::HashSet;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

use crate::{
    chunks::registry::ChunkRegistry,
    scene::{assets::MaterialType, ChunkOp},
};
use crate::{
    chunks::{registry::ChunkStatus, tasks::GenerateChunk},
    common::{
        chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F32},
        world::{BlockPosition, BlockVisibility, ChunkBlockId, WorldPosition},
    },
};
use crate::{
    fake_client::FakeClient,
    render::mesh::{bevy_mesh_greedy_quads, bevy_mesh_visible_block_faces},
};

use super::assets::Registry;

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

pub fn process_ops(
    mut commands: Commands,
    mut chunks: ResMut<ChunkRegistry>,
    client: Res<FakeClient>,
    assets: Res<Registry>,
    mut scene: ResMut<crate::scene::Scene>,
    mut meshes: ResMut<Assets<Mesh>>,
    registry: Res<Registry>,
) {
    let zoomf = 2f64.powf(scene.zoom_level as f64);
    let mut queued_generations = vec![];
    for _ in 0..CHUNK_OP_RATE {
        let Some(op) = scene.ops.pop_front() else {
            if scene.is_processing_ops {
                tracing::info!("Finished processing chunk ops");
                scene.is_processing_ops = false;
            }
            return;
        };

        let cpos = match op {
            ChunkOp::Load(cpos) => match chunks.get_status(&scene.zoom_level, &cpos) {
                Some(status) => match status {
                    ChunkStatus::Present(_) => {
                        tracing::debug!(?cpos, "Will render chunk");
                        cpos
                    }
                    ChunkStatus::Generating => {
                        queued_generations.push(ChunkOp::Load(cpos));
                        continue;
                    }
                },
                None => {
                    chunks.set_status(&scene.zoom_level, &cpos, ChunkStatus::Generating);
                    let thread_pool = AsyncComputeTaskPool::get();
                    let worldgen = client.world.clone();
                    let zoom_level = scene.zoom_level;
                    let task = thread_pool.spawn(async move {
                        (zoom_level, cpos, worldgen.write().unwrap().get(&cpos, zoomf))
                    });
                    commands.spawn(GenerateChunk(task));
                    queued_generations.push(ChunkOp::Load(cpos));
                    continue;
                }
            },
            ChunkOp::Unload(cpos) => {
                if let Some(eids) = scene.loaded.get(&cpos) {
                    tracing::debug!(?cpos, "Unloading chunk");
                    eids.iter().for_each(|physical_eid| {
                        commands.entity(*physical_eid).despawn();
                    });
                    tracing::debug!(?cpos, "Chunk unloaded");
                    scene.loaded.remove(&cpos);
                }
                continue;
            }
        };

        let Chunk::Unpacked(chunk) = chunks.get_mut(
            &scene.zoom_level,
            &cpos,
            client.as_ref(),
            &registry.block_mappings,
        ) else {
            tracing::debug!(?cpos, "Empty chunk");
            continue;
        };
        let opaque_mat = assets.get_material(MaterialType::DenseOpaque);
        let translucent_mat = assets.get_material(MaterialType::Translucent);
        let neighbor_faces = chunks.get_neighboring_faces_mut(
            &scene.zoom_level,
            &cpos,
            client.as_ref(),
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
        let mut eids = HashSet::new();
        for (mesh, material) in loads {
            let eid = commands
                .spawn((PbrBundle {
                    mesh: meshes.add(mesh),
                    material,
                    transform,
                    ..default()
                },))
                .id();

            eids.insert(eid);
        }

        tracing::debug!(?cpos, ?eids, "Chunk loaded");
        scene.loaded.insert(cpos, eids);
    }
    for queued_generation in queued_generations.into_iter() {
        // prioritize rendering chunks we queued to generate this run
        scene.ops.push_front(queued_generation);
    }
}
