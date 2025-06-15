use std::ops::Deref;
use std::sync::Arc;

use bevy::prelude::*;
use infinigen_common::mesh::shapes::EMPTY_CHUNK_FACES;
use infinigen_common::world::{ChunkPosition, Direction};
use infinigen_common::zoom::ZoomLevel;
use strum::IntoEnumIterator;

use super::{MeshInfo, MeshStatus, Meshes};
use crate::mesh::utils::{bevy_mesh_greedy_quads, bevy_mesh_visible_block_faces};
use crate::registry::BlockRegistry;
use crate::world::events::GenerateChunkRequest;
use crate::world::{ChunkStatus, World};

#[derive(Event)]
pub struct MeshChunkRequest {
    pub chunk_position: ChunkPosition,
    pub zoom_level: ZoomLevel,
}

#[derive(Event)]
pub struct MeshChunkRerequest {
    pub chunk_position: ChunkPosition,
    pub zoom_level: ZoomLevel,
}

pub fn handle_mesh_chunk_rerequests(
    mut mesh_chunk_rerequests: EventReader<MeshChunkRerequest>,
    mut mesh_chunk_requests: EventWriter<MeshChunkRequest>,
) {
    for (
        MeshChunkRerequest {
            chunk_position,
            zoom_level,
        },
        _,
    ) in mesh_chunk_rerequests.par_read()
    {
        mesh_chunk_requests.write(MeshChunkRequest {
            chunk_position: *chunk_position,
            zoom_level: *zoom_level,
        });
    }
}

pub fn handle_mesh_chunk_requests(
    mut mesh_chunk_requests: EventReader<MeshChunkRequest>,
    mut mesh_chunk_rerequests: EventWriter<MeshChunkRerequest>,
    mut generate_chunk_reqs: EventWriter<GenerateChunkRequest>,
    world: Res<World>,
    mut meshes: ResMut<Meshes>,
    registry: Res<BlockRegistry>,
) {
    for (
        MeshChunkRequest {
            chunk_position,
            zoom_level,
        },
        _,
    ) in mesh_chunk_requests.par_read()
    {
        let Some(status) = world.cache.get(&(*chunk_position, *zoom_level)) else {
            // chunk not available yet, request generation and check to mesh later
            generate_chunk_reqs.write(GenerateChunkRequest {
                chunk_position: *chunk_position,
                zoom_level: *zoom_level,
            });
            mesh_chunk_rerequests.write(MeshChunkRerequest {
                chunk_position: *chunk_position,
                zoom_level: *zoom_level,
            });
            continue;
        };
        let chunk_info = match status {
            ChunkStatus::Generating => {
                mesh_chunk_rerequests.write(MeshChunkRerequest {
                    chunk_position: *chunk_position,
                    zoom_level: *zoom_level,
                });
                continue;
            }
            ChunkStatus::Generated(chunk_info) => chunk_info.deref(),
            ChunkStatus::Empty => {
                meshes
                    .meshes
                    .insert((*chunk_position, *zoom_level), MeshStatus::Empty);
                continue;
            }
        };
        let mut neighbours = vec![];
        let mut all_neighbours_present = true;
        for (dir, neighbour_cpos) in get_neighbour_cposes(chunk_position) {
            match world.cache.get(&(neighbour_cpos, *zoom_level)) {
                Some(status) => match status {
                    ChunkStatus::Generating => {
                        all_neighbours_present = false;
                        continue;
                    }
                    ChunkStatus::Generated(_) | ChunkStatus::Empty => {
                        neighbours.push((dir, status))
                    }
                },
                None => {
                    all_neighbours_present = false;
                    // request chunk generation but not mesh, as we might not need it
                    // mesh requests should be driven by the active scene
                    generate_chunk_reqs.write(GenerateChunkRequest {
                        chunk_position: neighbour_cpos,
                        zoom_level: *zoom_level,
                    });
                    continue;
                }
            }
        }
        if !all_neighbours_present {
            // not all neighbours available yet, check again later
            mesh_chunk_rerequests.write(MeshChunkRerequest {
                chunk_position: *chunk_position,
                zoom_level: *zoom_level,
            });
            continue;
        }

        let mut neighbour_faces = EMPTY_CHUNK_FACES;
        for (dir, neighbour) in neighbours.into_iter() {
            let opposite = dir.opposite();
            let faces = match neighbour {
                ChunkStatus::Generated(chunk_info) => chunk_info.faces,
                ChunkStatus::Empty => EMPTY_CHUNK_FACES,
                ChunkStatus::Generating => unreachable!(),
            };
            neighbour_faces[dir] = faces[opposite];
        }

        let mut mesh_info = MeshInfo::default();

        for translucent in chunk_info.translucents.iter() {
            if let Some(translucent_mesh) = bevy_mesh_greedy_quads(
                translucent,
                &neighbour_faces,
                &registry.appearances,
                &registry.definitions,
            ) {
                mesh_info.translucents.push(translucent_mesh);
            }
        }

        mesh_info.opaque = bevy_mesh_visible_block_faces(
            &chunk_info.opaque,
            &neighbour_faces,
            &registry.appearances,
            &registry.definitions,
        );

        if mesh_info.opaque.is_none() && mesh_info.translucents.is_empty() {
            // TODO: can this happen?
            meshes
                .meshes
                .insert((*chunk_position, *zoom_level), MeshStatus::Empty);
            continue;
        }

        meshes.meshes.insert(
            (*chunk_position, *zoom_level),
            MeshStatus::Meshed(Arc::new(mesh_info)),
        );
    }
}

pub fn get_neighbour_cposes(
    position: &ChunkPosition,
) -> impl Iterator<Item = (Direction, ChunkPosition)> + '_ {
    Direction::iter().map(|dir| {
        let normal: [i32; 3] = dir.into();
        (
            dir,
            ChunkPosition {
                x: position.x + normal[0],
                y: position.y + normal[1],
                z: position.z + normal[2],
            },
        )
    })
}
