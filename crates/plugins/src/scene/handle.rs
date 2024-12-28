use bevy::prelude::*;
use infinigen_common::chunks::CHUNK_SIZE_F32;
use infinigen_common::world::WorldPosition;

use super::{LoadedChunk, UnloadChunkOpEvent};
use crate::assets::blocks::{BlockRegistry, MaterialType};
use crate::chunks::registry::{ChunkRegistry, ChunkStatus};
use crate::chunks::tasks::RequestChunkEvent;
use crate::scene::utils::{bevy_mesh_greedy_quads, bevy_mesh_visible_block_faces};
use crate::world::World;

// bigger chunks means go slower to prevent lag/stutter
const CHUNK_OP_RATE: usize = (32. * (32. / CHUNK_SIZE_F32)) as usize;

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

                let chunk_info = chunk_info.clone();

                let neighbor_faces = chunks.get_neighboring_faces_mut(
                    scene_zoom_level,
                    &cpos,
                    world.as_ref(), // !!!!
                    &registry.definitions,
                );

                let opaque = chunk_info.opaque.as_ref();
                let translucents = &chunk_info.translucents;

                let mut loads = vec![];

                let block_textures = registry.get_appearances();

                for translucent in translucents.iter() {
                    if let Some(translucent_mesh) = bevy_mesh_greedy_quads(
                        translucent,
                        &neighbor_faces,
                        block_textures,
                        &registry.definitions,
                    ) {
                        loads.push((
                            translucent_mesh,
                            registry.get_material(MaterialType::Translucent),
                        ));
                    }
                }

                if let Some(opaque_mesh) = bevy_mesh_visible_block_faces(
                    opaque,
                    &neighbor_faces,
                    block_textures,
                    &registry.definitions,
                ) {
                    loads.push((
                        opaque_mesh,
                        registry.get_material(MaterialType::DenseOpaque),
                    ));
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
