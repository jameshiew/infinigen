use std::collections::hash_map::Entry;

use ahash::{AHashMap, AHashSet};
use bevy::prelude::*;
use infinigen_common::chunks::CHUNK_SIZE_F32;
use infinigen_common::view;
use infinigen_common::world::{ChunkPosition, WorldPosition};

use crate::AppState;

mod handle;
pub mod setup;

const MAX_CHUNKS_TO_QUEUE_PER_FRAME: usize = 200;

#[derive(Component)]
pub struct LoadedChunk {
    pub cpos: ChunkPosition,
}

#[derive(Default, Resource)]
pub struct ChunkRequests {
    requests: AHashMap<ChunkPosition, SceneChunkStatus>,
}

#[derive(Default, Resource)]
pub struct PendingChunkLoads {
    chunks: Vec<ChunkPosition>,
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

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct SceneView {
    pub horizontal_view_distance: usize,
    pub vertical_view_distance: usize,
}

impl Default for SceneView {
    fn default() -> Self {
        Self {
            horizontal_view_distance: 4,
            vertical_view_distance: 4,
        }
    }
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct SceneZoom {
    // Zoom as a power of 2. e.g. if this is 0, then there will be no zoom.
    pub prev_zoom_level: i8,
    pub zoom_level: i8,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct SceneSettings {
    pub zoom_level: i8,
    pub horizontal_view_distance: usize,
    pub vertical_view_distance: usize,
}

#[derive(Debug, Message)]
pub struct UnloadChunkOpMessage(ChunkPosition);

pub const FAR: f32 = CHUNK_SIZE_F32 * 64.;

#[derive(Message)]
pub struct RefreshChunkOpsQueueMessage;

#[derive(Message)]
pub struct ReloadAllChunksMessage;

#[derive(Message)]
pub enum UpdateSettingsMessage {
    HorizontalViewDistance(usize),
    VerticalViewDistance(usize),
    ZoomLevel(i8),
}

pub fn handle_update_scene_view(
    mut scene_view: ResMut<SceneView>,
    mut scene_zoom: ResMut<SceneZoom>,
    mut update_msgs: MessageReader<UpdateSettingsMessage>,
    mut refresh_msgs: MessageWriter<RefreshChunkOpsQueueMessage>,
    mut reload_msgs: MessageWriter<ReloadAllChunksMessage>,
) {
    for msg in update_msgs.read() {
        match msg {
            UpdateSettingsMessage::HorizontalViewDistance(horizontal_view_distance) => {
                tracing::info!(
                    "Updating horizontal view distance from {} to {}",
                    scene_view.horizontal_view_distance,
                    horizontal_view_distance
                );
                scene_view.horizontal_view_distance = *horizontal_view_distance;
                refresh_msgs.write(RefreshChunkOpsQueueMessage);
            }
            UpdateSettingsMessage::VerticalViewDistance(vertical_view_distance) => {
                tracing::info!(
                    "Updating vertical view distance from {} to {}",
                    scene_view.vertical_view_distance,
                    vertical_view_distance
                );
                scene_view.vertical_view_distance = *vertical_view_distance;
                refresh_msgs.write(RefreshChunkOpsQueueMessage);
            }
            UpdateSettingsMessage::ZoomLevel(zoom_level) => {
                tracing::info!(
                    "Updating zoom level from {} to {}",
                    scene_zoom.zoom_level,
                    zoom_level
                );
                scene_zoom.prev_zoom_level = scene_zoom.zoom_level;
                scene_zoom.zoom_level = *zoom_level;
                reload_msgs.write(ReloadAllChunksMessage);
            }
        }
    }
}

#[derive(Message)]
pub struct UpdateSceneMessage;

pub fn check_if_should_update_scene(
    mut commands: Commands,
    mut scene_camera: ResMut<SceneCamera>,
    camera: Single<&Transform, With<Camera>>,
    mut chunk_requests: ResMut<ChunkRequests>,
    mut pending_loads: ResMut<PendingChunkLoads>,
    mut reload_msgs: MessageReader<ReloadAllChunksMessage>,
    mut refresh_msgs: MessageReader<RefreshChunkOpsQueueMessage>,
    mut update_scene_msgs: MessageWriter<UpdateSceneMessage>,
    loaded: Query<Entity, With<LoadedChunk>>,
) {
    let mut should_update = if refresh_msgs.read().next().is_some() {
        chunk_requests.clear();
        pending_loads.chunks.clear();
        true
    } else if reload_msgs.read().next().is_some() {
        chunk_requests.clear();
        pending_loads.chunks.clear();
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
    update_scene_msgs.write(UpdateSceneMessage);
}

pub fn update_scene(
    scene_view: Res<SceneView>,
    camera: Query<(&Transform, &Projection), With<Camera>>,
    mut pending_loads: ResMut<PendingChunkLoads>,
    mut unload_msgs: MessageWriter<UnloadChunkOpMessage>,
    mut update_scene_msgs: MessageReader<UpdateSceneMessage>,
    loaded: Query<&LoadedChunk>,
) -> Result {
    if update_scene_msgs.read().next().is_none() {
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
        scene_view.horizontal_view_distance,
        scene_view.vertical_view_distance,
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

    pending_loads.chunks = to_load;

    unload_msgs.write_batch(to_unload.into_iter().map(UnloadChunkOpMessage));
    Ok(())
}

/// Process pending chunk loads in batches to prevent freezing.
pub fn process_pending_chunk_loads(
    mut chunk_requests: ResMut<ChunkRequests>,
    mut pending_loads: ResMut<PendingChunkLoads>,
) {
    if pending_loads.chunks.is_empty() {
        return;
    }

    let batch_size = MAX_CHUNKS_TO_QUEUE_PER_FRAME.min(pending_loads.chunks.len());
    let to_add: Vec<_> = pending_loads.chunks.drain(..batch_size).collect();

    for cpos in to_add {
        chunk_requests.request_load(cpos);
    }
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing scene plugin");
        app.init_resource::<SceneView>()
            .init_resource::<SceneCamera>()
            .init_resource::<SceneZoom>()
            .init_resource::<ChunkRequests>()
            .init_resource::<PendingChunkLoads>()
            .add_systems(
                OnEnter(AppState::MainGame),
                (setup::setup_lighting, setup::setup),
            )
            .add_message::<UpdateSettingsMessage>()
            .add_message::<ReloadAllChunksMessage>()
            .add_message::<RefreshChunkOpsQueueMessage>()
            .add_message::<UnloadChunkOpMessage>()
            .add_message::<UpdateSceneMessage>()
            .add_systems(
                FixedUpdate,
                ((
                    (
                        handle::process_load_requested,
                        handle::process_mesh_requested,
                        handle::process_spawn_requested,
                    )
                        .chain(),
                    handle::process_unload_chunk_ops.run_if(on_message::<UnloadChunkOpMessage>),
                )
                    .run_if(in_state(AppState::MainGame)),),
            )
            .add_systems(
                Update,
                ((
                    handle_update_scene_view.run_if(on_message::<UpdateSettingsMessage>),
                    check_if_should_update_scene,
                    update_scene.run_if(on_message::<UpdateSceneMessage>),
                    process_pending_chunk_loads,
                )
                    .chain())
                .run_if(in_state(AppState::MainGame)),
            );
    }
}
