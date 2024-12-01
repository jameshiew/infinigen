use std::cmp::Ordering;
use std::collections::VecDeque;
use std::f32::consts::PI;

use bevy::prelude::*;
use nalgebra::{Matrix4, Vector3};
use nalgebra::{Perspective3, Quaternion, UnitQuaternion};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::settings::{Config, DEFAULT_HORIZONTAL_VIEW_DISTANCE, DEFAULT_VERTICAL_VIEW_DISTANCE};
use crate::AppState;
use infinigen_common::chunks::CHUNK_SIZE_F32;
use infinigen_common::world::{ChunkPosition, WorldPosition};

mod handle;
pub mod lights;
pub mod visible_chunks;

/// Holds details of the currently rendered scene.
#[derive(Debug, Default, Resource)]
pub struct Scene {
    /// Loaded chunks and their entities.
    pub loaded: FxHashMap<ChunkPosition, FxHashSet<Entity>>,
}

#[derive(Resource, Default)]
pub struct LoadOps {
    pub deque: VecDeque<RequestChunkOp>,
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
            hview_distance: DEFAULT_HORIZONTAL_VIEW_DISTANCE,
            vview_distance: DEFAULT_VERTICAL_VIEW_DISTANCE,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct SceneZoom {
    // Zoom as a power of 2. e.g. if this is 0, then there will be no zoom.
    pub prev_zoom_level: i8,
    pub zoom_level: i8,
}

// Prioritizing which chunks to load is important so we can't use events
#[derive(Debug)]
pub struct RequestChunkOp(ChunkPosition);

#[derive(Debug, Event)]
pub struct UnloadChunkOpEvent(ChunkPosition);

pub const FAR: f32 = CHUNK_SIZE_F32 * 64.;

pub fn init_scene_from_config(
    mut scene: ResMut<Scene>,
    mut scene_view: ResMut<SceneView>,
    mut scene_zoom: ResMut<SceneZoom>,
    config: Res<Config>,
) {
    scene_view.hview_distance = config.hview_distance;
    scene_view.vview_distance = config.vview_distance;
    scene_zoom.prev_zoom_level = config.zoom_level;
    scene_zoom.zoom_level = config.zoom_level;

    // we expect roughly this many chunks to be loaded initially (a cylinder centred around the player)
    let initial_capacity = (PI * scene_view.hview_distance.pow(2) as f32)
        * (scene_view.vview_distance as f32 * 2. + 1.);
    let initial_capacity = initial_capacity.floor() as usize;
    tracing::info!(
        ?config.hview_distance,
        ?config.vview_distance,
        ?initial_capacity,
        "Setting initial capacity for loaded chunks"
    );
    // TODO: spawn load ops for each chunk that will be in the initial view, then camera_cpos needn't be an option
    scene.loaded = FxHashMap::default();
}

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
    mut camera: Query<&mut Transform, With<Camera>>,
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
                refresh_evs.send(RefreshChunkOpsQueueEvent);
            }
            UpdateSettingsEvent::VerticalViewDistance(vview_distance) => {
                tracing::info!(
                    "Updating vertical view distance from {} to {}",
                    scene_view.vview_distance,
                    vview_distance
                );
                scene_view.vview_distance = *vview_distance;
                refresh_evs.send(RefreshChunkOpsQueueEvent);
            }
            UpdateSettingsEvent::ZoomLevel(zoom_level) => {
                tracing::info!(
                    "Updating zoom level from {} to {}",
                    scene_zoom.zoom_level,
                    zoom_level
                );
                scene_zoom.prev_zoom_level = scene_zoom.zoom_level;
                scene_zoom.zoom_level = *zoom_level;

                let mut camera = camera.single_mut();
                let dzoom = (scene_zoom.zoom_level - scene_zoom.prev_zoom_level) as f32;
                camera.translation.x *= 2f32.powf(dzoom);
                camera.translation.y *= 2f32.powf(dzoom);
                camera.translation.z *= 2f32.powf(dzoom);
                reload_evs.send(ReloadAllChunksEvent);
            }
        }
    }
}

#[derive(Event)]
pub struct UpdateSceneEvent;

pub fn check_if_should_update_scene(
    mut commands: Commands,
    mut scene: ResMut<Scene>,
    mut scene_camera: ResMut<SceneCamera>,
    camera: Query<&Transform, With<Camera>>,
    mut reload_evs: EventReader<ReloadAllChunksEvent>,
    mut refresh_evs: EventReader<RefreshChunkOpsQueueEvent>,
    mut update_scene_evs: EventWriter<UpdateSceneEvent>,
) {
    let mut should_update = false;
    if refresh_evs.read().next().is_some() {
        should_update = true;
    }
    if reload_evs.read().next().is_some() {
        tracing::info!("Reloading all chunks");
        for (_, eids) in scene.loaded.drain() {
            eids.iter().for_each(|physical_eid| {
                commands.entity(*physical_eid).despawn();
            });
        }
        should_update = true;
    }

    let camera = camera.single();
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
    update_scene_evs.send(UpdateSceneEvent);
}

pub fn update_scene(
    scene_view: Res<SceneView>,
    scene: Res<Scene>,
    camera: Query<(&Transform, &Projection), With<Camera>>,
    mut load_ops: ResMut<LoadOps>,
    mut unload_evs: EventWriter<UnloadChunkOpEvent>,
    mut update_scene_evs: EventReader<UpdateSceneEvent>,
) {
    if update_scene_evs.read().next().is_none() {
        return;
    }
    load_ops.deque.clear();

    let (camera, projection) = camera.single();
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
    let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
        camera.rotation.w,
        camera.rotation.x,
        camera.rotation.y,
        camera.rotation.z,
    ));
    let translation = Vector3::new(
        camera.translation.x,
        camera.translation.y,
        camera.translation.z,
    );

    // Create a Perspective projection matrix using nalgebra's Perspective3
    let persp_proj = Perspective3::new(aspect_ratio, fov, near, far);
    let projection_matrix: Matrix4<f32> = *persp_proj.as_matrix();

    // Compute the View matrix
    let view_matrix: Matrix4<f32> =
        Matrix4::from(rotation.to_rotation_matrix()).append_translation(&-translation);

    // Compute the frustum planes from the combined matrix
    let combined_matrix = projection_matrix * view_matrix;
    let frustum_planes = visible_chunks::compute_frustum_planes(&combined_matrix);

    let chunks_within_render_distance: FxHashSet<_> = visible_chunks::in_distance(
        &current_cpos,
        scene_view.hview_distance,
        scene_view.vview_distance,
    )
    .collect();

    let already_loaded_or_loading: FxHashSet<_> = scene.loaded.keys().cloned().collect();

    let mut to_load: Vec<_> = chunks_within_render_distance
        .difference(&already_loaded_or_loading)
        .collect();

    // nearest chunks first
    to_load.sort_unstable_by_key(|&c| {
        let dx = c.x - current_cpos.x;
        let dy = c.y - current_cpos.y;
        let dz = c.z - current_cpos.z;
        dx * dx + dy * dy + dz * dz
    });
    // chunks within view frustum first
    to_load.sort_unstable_by(|&c1, &c2| {
        let in_frustum1 = visible_chunks::check_chunk_in_frustum(c1, &frustum_planes);
        let in_frustum2 = visible_chunks::check_chunk_in_frustum(c2, &frustum_planes);

        if in_frustum1 && !in_frustum2 {
            Ordering::Less
        } else if !in_frustum1 && in_frustum2 {
            Ordering::Greater
        } else {
            let dx1 = c1.x - current_cpos.x;
            let dy1 = c1.y - current_cpos.y;
            let dz1 = c1.z - current_cpos.z;
            let dist1 = dx1 * dx1 + dy1 * dy1 + dz1 * dz1;

            let dx2 = c2.x - current_cpos.x;
            let dy2 = c2.y - current_cpos.y;
            let dz2 = c2.z - current_cpos.z;
            let dist2 = dx2 * dx2 + dy2 * dy2 + dz2 * dz2;

            dist1.cmp(&dist2)
        }
    });

    to_load.iter().for_each(|&cpos| {
        let cpos = cpos.to_owned();
        load_ops.deque.push_back(RequestChunkOp(cpos))
    });
    // the order in which chunks are unloaded is not so important
    let to_unload = already_loaded_or_loading.difference(&chunks_within_render_distance);
    to_unload.for_each(|cpos| {
        let cpos = cpos.to_owned();
        unload_evs.send(UnloadChunkOpEvent(cpos));
    });
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing scene plugin");
        app.init_resource::<Scene>()
            .init_resource::<SceneView>()
            .init_resource::<SceneCamera>()
            .init_resource::<SceneZoom>()
            .init_resource::<LoadOps>()
            .add_systems(Startup, (lights::setup, init_scene_from_config))
            .add_event::<UpdateSettingsEvent>()
            .add_event::<ReloadAllChunksEvent>()
            .add_event::<RefreshChunkOpsQueueEvent>()
            .add_event::<UnloadChunkOpEvent>()
            .add_event::<UpdateSceneEvent>()
            .add_systems(
                Update,
                (
                    check_if_should_update_scene.run_if(in_state(AppState::MainGame)),
                    update_scene
                        .run_if(in_state(AppState::MainGame))
                        .after(check_if_should_update_scene),
                    handle::process_load_chunk_ops.run_if(in_state(AppState::MainGame)),
                    handle::process_unload_chunk_ops.run_if(in_state(AppState::MainGame)),
                    handle_update_scene_view.run_if(in_state(AppState::MainGame)),
                ),
            );
    }
}
