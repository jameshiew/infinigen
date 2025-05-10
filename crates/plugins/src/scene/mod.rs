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
pub struct SceneChunks {
    requests: AHashMap<ChunkPosition, SceneChunkStatus>,
}

#[derive(Clone, Copy)]
pub enum SceneChunkStatus {
    LoadRequested,
    MeshRequested,
    SpawnRequested,
}

impl SceneChunks {
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

#[derive(Debug, Event)]
pub struct UnloadChunkOpEvent(ChunkPosition);

pub const FAR: f32 = CHUNK_SIZE_F32 * 64.;

#[derive(Event)]
pub struct RefreshChunkOpsQueueEvent;

#[derive(Event)]
pub struct ReloadAllChunksEvent;

#[derive(Event)]
pub enum UpdateSettingsEvent {
    HorizontalViewDistance(usize),
    VerticalViewDistance(usize),
    ZoomLevel(i8),
}

pub fn handle_update_scene_view(
    mut scene_view: ResMut<SceneView>,
    mut scene_zoom: ResMut<SceneZoom>,
    mut camera: Single<&mut Transform, With<Camera>>,
    mut update_evs: EventReader<UpdateSettingsEvent>,
    mut refresh_evs: EventWriter<RefreshChunkOpsQueueEvent>,
    mut reload_evs: EventWriter<ReloadAllChunksEvent>,
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

                let dzoom = (scene_zoom.zoom_level - scene_zoom.prev_zoom_level) as f32;
                camera.translation.x *= 2f32.powf(dzoom);
                camera.translation.y *= 2f32.powf(dzoom);
                camera.translation.z *= 2f32.powf(dzoom);
                reload_evs.write(ReloadAllChunksEvent);
            }
        }
    }
}

#[derive(Event)]
pub struct UpdateSceneEvent;

#[allow(clippy::too_many_arguments)]
pub fn check_if_should_update_scene(
    mut commands: Commands,
    mut scene_camera: ResMut<SceneCamera>,
    camera: Single<&Transform, With<Camera>>,
    mut scene_chunks: ResMut<SceneChunks>,
    mut reload_evs: EventReader<ReloadAllChunksEvent>,
    mut refresh_evs: EventReader<RefreshChunkOpsQueueEvent>,
    mut update_scene_evs: EventWriter<UpdateSceneEvent>,
    loaded: Query<Entity, With<LoadedChunk>>,
) {
    let mut should_update = false;
    if refresh_evs.read().next().is_some() {
        scene_chunks.clear();
        should_update = true;
    }
    if reload_evs.read().next().is_some() {
        scene_chunks.clear();
        tracing::info!("Reloading all chunks");
        for loaded_chunk in loaded.iter() {
            commands.entity(loaded_chunk).despawn();
        }
        should_update = true;
    }

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

// Updated `update_scene` system that uses the new helper function
pub fn update_scene(
    scene_view: Res<SceneView>,
    camera: Query<(&Transform, &Projection), With<Camera>>,
    mut scene_chunks: ResMut<SceneChunks>,
    mut unload_evs: EventWriter<UnloadChunkOpEvent>,
    mut update_scene_evs: EventReader<UpdateSceneEvent>,
    loaded: Query<&LoadedChunk>,
) {
    if update_scene_evs.read().next().is_none() {
        return;
    }
    tracing::trace!("Updating scene");

    let (camera, projection) = camera.single().unwrap();
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
        scene_chunks.request_load(cpos);
    }

    unload_evs.write_batch(to_unload.into_iter().map(UnloadChunkOpEvent));
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing scene plugin");
        app.init_resource::<SceneView>()
            .init_resource::<SceneCamera>()
            .init_resource::<SceneZoom>()
            .init_resource::<SceneChunks>()
            .add_systems(
                OnEnter(AppState::MainGame),
                (setup::setup_lighting, setup::setup),
            )
            .add_event::<UpdateSettingsEvent>()
            .add_event::<ReloadAllChunksEvent>()
            .add_event::<RefreshChunkOpsQueueEvent>()
            .add_event::<UnloadChunkOpEvent>()
            .add_event::<UpdateSceneEvent>()
            .add_systems(
                FixedUpdate,
                ((
                    (
                        handle::process_load_requested,
                        handle::process_mesh_requested,
                        handle::process_spawn_requested,
                    )
                        .chain(),
                    handle::process_unload_chunk_ops.run_if(on_event::<UnloadChunkOpEvent>),
                )
                    .run_if(in_state(AppState::MainGame)),),
            )
            .add_systems(
                Update,
                ((
                    handle_update_scene_view.run_if(on_event::<UpdateSettingsEvent>),
                    check_if_should_update_scene,
                    update_scene.run_if(on_event::<UpdateSceneEvent>),
                )
                    .chain())
                .run_if(in_state(AppState::MainGame)),
            );
    }
}
