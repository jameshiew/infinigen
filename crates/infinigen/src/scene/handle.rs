use bevy::prelude::*;
use infinigen_common::blocks::BlockVisibility;
use infinigen_common::chunks::{Array3Chunk, Chunk, CHUNK_SIZE, CHUNK_SIZE_F32};
use infinigen_common::world::{BlockPosition, MappedBlockID, WorldPosition};

use super::{LoadedChunk, UnloadChunkOpEvent};
use crate::assets::blocks::{BlockRegistry, MaterialType};
use crate::chunks::registry::{ChunkRegistry, ChunkStatus};
use crate::chunks::tasks::RequestChunkEvent;
use crate::mesh::utils::{bevy_mesh_greedy_quads, bevy_mesh_visible_block_faces};
use crate::world::World;

// bigger chunks means go slower to prevent lag/stutter
const CHUNK_OP_RATE: usize = (16. * (32. / CHUNK_SIZE_F32)) as usize;

/// Split out blocks from this chunk.
pub fn split(mut chunk: Array3Chunk, chunk_block_id: MappedBlockID) -> (Array3Chunk, Chunk) {
    let mut split_out = Array3Chunk::default();
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
        Chunk::Array3(Box::new(split_out))
    } else {
        Chunk::Empty
    };
    (chunk, split_out)
}

pub fn process_unload_chunk_ops(
    mut commands: Commands,
    mut unload_evs: EventReader<crate::scene::UnloadChunkOpEvent>,
    loaded: Query<(Entity, &LoadedChunk)>,
) {
    for (UnloadChunkOpEvent(cpos), _) in unload_evs.par_read() {
        for (eid, LoadedChunk { cpos: loaded_cpos }) in loaded.iter() {
            if loaded_cpos == cpos {
                commands.entity(eid).despawn_recursive();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_load_chunk_ops(
    mut commands: Commands,
    mut chunks: ResMut<ChunkRegistry>,
    world: Res<World>,
    mut load_ops: ResMut<crate::scene::LoadChunkRequests>,
    scene_zoom: Res<crate::scene::SceneZoom>,
    mut meshes: ResMut<Assets<Mesh>>,
    registry: Res<BlockRegistry>,
    mut request_chunk_evs: EventWriter<RequestChunkEvent>,
) {
    let scene_zoom_level = scene_zoom.zoom_level.into();
    for _ in 0..CHUNK_OP_RATE {
        let Some(cpos) = load_ops.pop_front() else {
            return;
        };

        match chunks.get_status(scene_zoom_level, &cpos) {
            None => {
                tracing::trace!(?cpos, "Requesting chunk generation");
                request_chunk_evs.send(RequestChunkEvent {
                    zoom_level: scene_zoom_level,
                    position: cpos,
                });
            }
            Some(ChunkStatus::Generated(chunk_info)) => {
                tracing::trace!(?cpos, "Spawning chunk");

                let chunk = &chunk_info.chunk;
                let mut opaque_chunk = chunk.to_owned();
                let opaque_mat = registry.get_material(MaterialType::DenseOpaque);
                let translucent_mat = registry.get_material(MaterialType::Translucent);
                let neighbor_faces = chunks.get_neighboring_faces_mut(
                    scene_zoom_level,
                    &cpos,
                    world.as_ref(), // !!!!
                    &registry.block_mappings,
                );

                let mut loads = Vec::with_capacity(1); // most common case - only one mesh needed, for opaque blocks in chunk

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
                    let (remaining, translucent_chunk) =
                        split(*opaque_chunk, translucent_chunk_block_id);
                    opaque_chunk = Box::new(remaining);

                    if let Chunk::Array3(translucent_chunk) = translucent_chunk {
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
                    tracing::trace!(?cpos, "Occluded chunk");
                    continue;
                }

                let wpos: WorldPosition = (&cpos).into();
                let transform = Transform::from_xyz(wpos.x, wpos.y, wpos.z);
                // TODO: the above meshing stuff should be async also

                let cid = commands
                    .spawn((
                        Name::new("Chunk"),
                        LoadedChunk { cpos },
                        Visibility::default(),
                        transform,
                    ))
                    .id();
                for (mesh, material) in loads {
                    commands
                        .spawn((
                            Name::new("Chunk mesh"),
                            Mesh3d(meshes.add(mesh)),
                            MeshMaterial3d(material),
                        ))
                        .set_parent(cid);
                }
            }
            _ => continue,
        }
    }
}
