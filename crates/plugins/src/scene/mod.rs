use std::collections::hash_map::Entry;

use ahash::{AHashMap, AHashSet};
use bevy::prelude::*;
use infinigen_common::chunks::CHUNK_SIZE_F32;
use infinigen_common::view;
use infinigen_common::world::{ChunkPosition, WorldPosition};

use crate::AppState;

mod handle;
pub mod setup;

#[derive(Component)]
pub struct LoadedChunk {
    pub cpos: ChunkPosition,
}

#[derive(Default, Resource)]
pub struct ChunkRequests {
    requests: AHashMap<ChunkPosition, SceneChunkStatus>,
}

#[derive(Clone, Copy)]
pub enum SceneChunkStatus {
    LoadRequested,
    MeshRequested,
    SpawnRequested,
}

impl ChunkRequests {
    pub fn add(&mut self, cpos: ChunkPosition, request: SceneChunkStatus) {
        self.requests.insert(cpos, request);
    }

    pub fn request_load(&mut self, cpos: ChunkPosition) {
        match self.requests.entry(cpos) {
            Entry::Occupied(occupied_entry) => match occupied_entry.get() {
                SceneChunkStatus::LoadRequested
                | SceneChunkStatus::MeshRequested
                | SceneChunkStatus::SpawnRequested => (),
            },
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(SceneChunkStatus::LoadRequested);
            }
        };
    }

    pub fn remove(&mut self, cpos: ChunkPosition) {
        self.requests.remove(&cpos);
    }

    pub fn get_priority_requests(&self, n: usize) -> Vec<(ChunkPosition, SceneChunkStatus)> {
        // TODO: should sort by priority to render i.e. closest and in view frustum first
        let v: Vec<_> = self
            .requests
            .iter()
            .take(n)
            .map(|(cpos, status)| (*cpos, *status))
            .collect();
        v
    }

    pub fn all_statuses(&self) -> Vec<(ChunkPosition, SceneChunkStatus)> {
        self.get_priority_requests(usize::MAX)
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn clear(&mut self) {
        self.requests.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

#[derive(Debug, Default, Resource)]
pub struct SceneCamera {
    /// The current chunk the player is located in.
    cpos: Option<ChunkPosition>,
}

#[derive(Debug, Resource)]
pub struct SceneView {
    pub hview_distance: usize,
    pub vview_distance: usize,
}

impl Default for SceneView {
    fn default() -> Self {
        Self {
            hview_distance: 4,
            vview_distance: 4,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct SceneZoom {
    // Zoom as a power of 2. e.g. if this is 0, then there will be no zoom.
    pub prev_zoom_level: i8,
    pub zoom_level: i8,
}

#[derive(Resource)]
pub struct SceneSettings {
    pub zoom_level: i8,
    pub hview_distance: usize,
    pub vview_distance: usize,
}

#[derive(Debug, Message)]
pub struct UnloadChunkOpEvent(ChunkPosition);

pub const FAR: f32 = CHUNK_SIZE_F32 * 64.;

#[derive(Message)]
pub struct RefreshChunkOpsQueueEvent;

#[derive(Message)]
pub struct ReloadAllChunksEvent;

#[derive(Message)]
pub enum UpdateSettingsEvent {
    HorizontalViewDistance(usize),
    VerticalViewDistance(usize),
    ZoomLevel(i8),
}

pub fn handle_update_scene_view(
    mut scene_view: ResMut<SceneView>,
    mut scene_zoom: ResMut<SceneZoom>,
    mut update_evs: MessageReader<UpdateSettingsEvent>,
    mut refresh_evs: MessageWriter<RefreshChunkOpsQueueEvent>,
    mut reload_evs: MessageWriter<ReloadAllChunksEvent>,
) {
    for ev in update_evs.read() {
        match ev {
            UpdateSettingsEvent::HorizontalViewDistance(hview_distance) => {
                tracing::info!(
                    "Updating horizontal view distance from {} to {}",
                    scene_view.hview_distance,
                    hview_distance
                );
                scene_view.hview_distance = *hview_distance;
                refresh_evs.write(RefreshChunkOpsQueueEvent);
            }
            UpdateSettingsEvent::VerticalViewDistance(vview_distance) => {
                tracing::info!(
                    "Updating vertical view distance from {} to {}",
                    scene_view.vview_distance,
                    vview_distance
                );
                scene_view.vview_distance = *vview_distance;
                refresh_evs.write(RefreshChunkOpsQueueEvent);
            }
            UpdateSettingsEvent::ZoomLevel(zoom_level) => {
                tracing::info!(
                    "Updating zoom level from {} to {}",
                    scene_zoom.zoom_level,
                    zoom_level
                );
                scene_zoom.prev_zoom_level = scene_zoom.zoom_level;
                scene_zoom.zoom_level = *zoom_level;
                reload_evs.write(ReloadAllChunksEvent);
            }
        }
    }
}

#[derive(Message)]
pub struct UpdateSceneEvent;

pub fn check_if_should_update_scene(
    mut commands: Commands,
    mut scene_camera: ResMut<SceneCamera>,
    camera: Single<&Transform, With<Camera>>,
    mut chunk_requests: ResMut<ChunkRequests>,
    mut reload_evs: MessageReader<ReloadAllChunksEvent>,
    mut refresh_evs: MessageReader<RefreshChunkOpsQueueEvent>,
    mut update_scene_evs: MessageWriter<UpdateSceneEvent>,
    loaded: Query<Entity, With<LoadedChunk>>,
) {
    let mut should_update = if refresh_evs.read().next().is_some() {
        chunk_requests.clear();
        true
    } else if reload_evs.read().next().is_some() {
        chunk_requests.clear();
        tracing::info!("Reloading all chunks");
        for loaded_chunk in loaded.iter() {
            commands.entity(loaded_chunk).despawn();
        }
        true
    } else {
        false
    };

    let current_cpos: ChunkPosition = WorldPosition {
        x: camera.translation.x,
        y: camera.translation.y,
        z: camera.translation.z,
    }
    .into();

    if Some(current_cpos) != scene_camera.cpos {
        let previous_cpos = scene_camera.cpos.replace(current_cpos);
        tracing::debug!(?previous_cpos, current_cpos = ?scene_camera.cpos, "Camera moved to new chunk");
        should_update = true;
    }
    if !should_update {
        return;
    }
    update_scene_evs.write(UpdateSceneEvent);
}

pub fn update_scene(
    scene_view: Res<SceneView>,
    camera: Query<(&Transform, &Projection), With<Camera>>,
    mut chunk_requests: ResMut<ChunkRequests>,
    mut unload_evs: MessageWriter<UnloadChunkOpEvent>,
    mut update_scene_evs: MessageReader<UpdateSceneEvent>,
    loaded: Query<&LoadedChunk>,
) -> Result {
    if update_scene_evs.read().next().is_none() {
        return Ok(());
    }
    tracing::trace!("Updating scene");

    let (camera, projection) = camera.single()?;
    let current_cpos: ChunkPosition = WorldPosition {
        x: camera.translation.x,
        y: camera.translation.y,
        z: camera.translation.z,
    }
    .into();

    let Projection::Perspective(projection) = projection else {
        unimplemented!("only perspective projection is supported right now")
    };

    let aspect_ratio = projection.aspect_ratio;
    let fov = projection.fov;
    let near = projection.near;
    let far = projection.far;

    let already_loaded: AHashSet<_> = loaded.iter().map(|l| l.cpos).collect();
    tracing::debug!(loaded = ?already_loaded.len(), "Chunks already loaded");

    let (to_load, to_unload) = view::compute_chunks_delta(
        current_cpos,
        scene_view.hview_distance,
        scene_view.vview_distance,
        [
            camera.translation.x,
            camera.translation.y,
            camera.translation.z,
        ],
        [
            camera.rotation.w,
            camera.rotation.x,
            camera.rotation.y,
            camera.rotation.z,
        ],
        aspect_ratio,
        fov,
        near,
        far,
        &already_loaded,
    );
    tracing::debug!(load = ?to_load.len(), unload = ?to_unload.len(), "Chunks to load/unload");

    for cpos in to_load.into_iter() {
        chunk_requests.request_load(cpos);
    }

    unload_evs.write_batch(to_unload.into_iter().map(UnloadChunkOpEvent));
    Ok(())
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing scene plugin");
        app.init_resource::<SceneView>()
            .init_resource::<SceneCamera>()
            .init_resource::<SceneZoom>()
            .init_resource::<ChunkRequests>()
            .add_systems(
                OnEnter(AppState::MainGame),
                (setup::setup_lighting, setup::setup),
            )
            .add_message::<UpdateSettingsEvent>()
            .add_message::<ReloadAllChunksEvent>()
            .add_message::<RefreshChunkOpsQueueEvent>()
            .add_message::<UnloadChunkOpEvent>()
            .add_message::<UpdateSceneEvent>()
            .add_systems(
                FixedUpdate,
                ((
                    (
                        handle::process_load_requested,
                        handle::process_mesh_requested,
                        handle::process_spawn_requested,
                    )
                        .chain(),
                    handle::process_unload_chunk_ops.run_if(on_message::<UnloadChunkOpEvent>),
                )
                    .run_if(in_state(AppState::MainGame)),),
            )
            .add_systems(
                Update,
                ((
                    handle_update_scene_view.run_if(on_message::<UpdateSettingsEvent>),
                    check_if_should_update_scene,
                    update_scene.run_if(on_message::<UpdateSceneEvent>),
                )
                    .chain())
                .run_if(in_state(AppState::MainGame)),
            );
    }
}
