use std::sync::Arc;

use ahash::AHashMap;
use bevy::prelude::*;
use events::{MeshChunkRequest, MeshChunkRerequest};
use infinigen_common::world::ChunkPosition;
use infinigen_common::zoom::ZoomLevel;

use crate::AppState;

pub mod events;
mod utils;

#[derive(Resource, Default)]
pub struct Meshes {
    pub meshes: AHashMap<(ChunkPosition, ZoomLevel), MeshStatus>,
}

#[derive(Debug, Default, Clone)]
pub struct MeshInfo {
    pub opaque: Option<Mesh>,
    pub translucents: Vec<Mesh>,
}

pub enum MeshStatus {
    Meshing,
    Meshed(Arc<MeshInfo>),
    Empty,
}

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing mesh plugin");
        app.init_resource::<Meshes>()
            .add_message::<MeshChunkRequest>()
            .add_message::<MeshChunkRerequest>()
            .add_systems(
                FixedUpdate,
                (
                    events::handle_mesh_chunk_requests.run_if(on_message::<MeshChunkRequest>),
                    events::handle_mesh_chunk_rerequests.run_if(on_message::<MeshChunkRerequest>),
                )
                    .chain()
                    .run_if(in_state(AppState::MainGame)),
            );
    }
}
