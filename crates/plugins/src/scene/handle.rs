use bevy::prelude::*;
use infinigen_common::blocks::BlockVisibility;
use infinigen_common::chunks::CHUNK_SIZE_F32;
use infinigen_common::world::WorldPosition;
use infinigen_common::zoom::ZoomLevel;

use super::{SceneChunkStatus, SceneChunks, UnloadChunkOpEvent};
use crate::assets::blocks::BlockRegistry;
use crate::mesh::events::MeshChunkRequest;
use crate::mesh::{MeshStatus, Meshes};
use crate::scene::LoadedChunk;

// bigger chunks means go slower to prevent lag/stutter
const MESH_SPAWN_RATE: usize = (128. * (32. / CHUNK_SIZE_F32)) as usize;
const CHUNK_REQUEST_RATE: usize = MESH_SPAWN_RATE * 5;

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

pub fn process_load_requested(
    mut scene_chunks: ResMut<SceneChunks>,
    scene_zoom: Res<crate::scene::SceneZoom>,
    meshes: Res<Meshes>,
    mut mesh_chunk_reqs: EventWriter<MeshChunkRequest>,
) {
    let zoom_level: ZoomLevel = scene_zoom.zoom_level.into();

    let statuses: Vec<_> = scene_chunks.all_statuses();
    let should_check: Vec<_> = statuses
        .into_iter()
        .filter_map(|(cpos, status)| {
            if matches!(status, SceneChunkStatus::LoadRequested) {
                Some(cpos)
            } else {
                None
            }
        })
        .take(CHUNK_REQUEST_RATE)
        .collect();

    for cpos in should_check {
        match meshes.meshes.get(&(cpos, zoom_level)) {
            None => {
                mesh_chunk_reqs.send(MeshChunkRequest {
                    chunk_position: cpos,
                    zoom_level,
                });
                scene_chunks.add(cpos, SceneChunkStatus::MeshRequested);
            }
            Some(MeshStatus::Meshing) => {
                scene_chunks.add(cpos, SceneChunkStatus::MeshRequested);
            }
            Some(MeshStatus::Empty) => {
                scene_chunks.remove(cpos);
            }
            Some(MeshStatus::Meshed(_)) => {
                scene_chunks.add(cpos, SceneChunkStatus::SpawnRequested);
            }
        }
    }
}

pub fn process_mesh_requested(
    mut scene_chunks: ResMut<SceneChunks>,
    scene_zoom: Res<crate::scene::SceneZoom>,
    meshes: Res<Meshes>,
) {
    let zoom_level: ZoomLevel = scene_zoom.zoom_level.into();

    let statuses: Vec<_> = scene_chunks.all_statuses();
    let should_check: Vec<_> = statuses
        .into_iter()
        .filter_map(|(cpos, status)| {
            if matches!(status, SceneChunkStatus::MeshRequested) {
                Some(cpos)
            } else {
                None
            }
        })
        .take(CHUNK_REQUEST_RATE)
        .collect();

    for cpos in should_check {
        match meshes.meshes.get(&(cpos, zoom_level)) {
            None => continue,
            Some(MeshStatus::Meshing) => continue,
            Some(MeshStatus::Empty) => {
                scene_chunks.remove(cpos);
            }
            Some(MeshStatus::Meshed(_)) => {
                scene_chunks.add(cpos, SceneChunkStatus::SpawnRequested);
            }
        }
    }
}

pub fn process_spawn_requested(
    mut scene_chunks: ResMut<SceneChunks>,
    mut commands: Commands,
    scene_zoom: Res<crate::scene::SceneZoom>,
    meshes: Res<Meshes>,
    registry: Res<BlockRegistry>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    let zoom_level: ZoomLevel = scene_zoom.zoom_level.into();
    let statuses: Vec<_> = scene_chunks.all_statuses();
    let should_spawn = statuses
        .into_iter()
        .filter_map(|(cpos, status)| {
            if matches!(status, SceneChunkStatus::SpawnRequested) {
                Some(cpos)
            } else {
                None
            }
        })
        .take(MESH_SPAWN_RATE);

    for cpos in should_spawn {
        let MeshStatus::Meshed(mesh_info) = meshes
            .meshes
            .get(&(cpos, zoom_level))
            .expect("must be able to get mesh")
        else {
            unreachable!("must be able to get mesh");
        };
        let wpos: WorldPosition = (&cpos).into();
        let transform = Transform::from_xyz(wpos.x, wpos.y, wpos.z);

        let chunk_entity = commands
            .spawn((
                Name::new(format!("Chunk {cpos:?}")),
                LoadedChunk { cpos },
                Transform::from_translation(transform.translation),
                Visibility::default(),
            ))
            .id();

        if let Some(ref opaque_mesh) = mesh_info.opaque {
            commands
                .spawn((
                    Name::new("Opaque mesh"),
                    Mesh3d(mesh_assets.add(opaque_mesh.clone())),
                    MeshMaterial3d(registry.get_material(BlockVisibility::Opaque)),
                    Transform::default(),
                    Visibility::default(),
                ))
                .set_parent(chunk_entity);
        }

        for trans_mesh in &mesh_info.translucents {
            commands
                .spawn((
                    Name::new("Translucent mesh"),
                    Mesh3d(mesh_assets.add(trans_mesh.clone())),
                    MeshMaterial3d(registry.get_material(BlockVisibility::Translucent)),
                    Transform::default(),
                    Visibility::default(),
                ))
                .set_parent(chunk_entity);
        }
        scene_chunks.remove(cpos);
    }
}
