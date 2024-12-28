//! Converts from our native mesh types to Bevy meshes

use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use indexed_priority_queue::DefaultMapIPQ;
use infinigen_common::chunks::Array3Chunk;
use infinigen_common::mesh::faces::{prepare_padded_chunk, BlockVisibilityChecker};
use infinigen_common::mesh::shapes::ChunkFace;
use infinigen_common::mesh::textures::BlockAppearances;
use infinigen_common::mesh::{mesh_chunk_greedy_quads, mesh_chunk_visible_block_faces, MeshInfo};
use infinigen_common::world::{ChunkPosition, Direction};
use linearize::StaticCopyMap;

pub fn to_bevy_mesh(
    MeshInfo {
        positions,
        normals,
        colors,
        uvs,
        indices,
    }: MeshInfo,
) -> Mesh {
    let mut render_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(colors),
    );
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    render_mesh.insert_indices(Indices::U32(indices));
    render_mesh
}

/// Returns a mesh of all visible block faces in the chunk.
/// adapted from <https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs>
pub fn bevy_mesh_visible_block_faces(
    chunk: &Array3Chunk,
    neighbor_faces: &StaticCopyMap<Direction, ChunkFace>,
    block_textures: &BlockAppearances,
    visibility_checker: impl BlockVisibilityChecker,
) -> Option<Mesh> {
    let samples = prepare_padded_chunk(chunk, neighbor_faces, visibility_checker);
    let mesh = mesh_chunk_visible_block_faces(&samples, block_textures);
    mesh.map(to_bevy_mesh)
}

/// Returns a mesh of quads in a chunk.
/// adapted from <https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs>
pub fn bevy_mesh_greedy_quads(
    chunk: &Array3Chunk,
    neighbor_faces: &StaticCopyMap<Direction, ChunkFace>,
    block_textures: &BlockAppearances,
    visibility_checker: impl BlockVisibilityChecker,
) -> Option<Mesh> {
    let samples = prepare_padded_chunk(chunk, neighbor_faces, visibility_checker);
    let mesh = mesh_chunk_greedy_quads(&samples, block_textures);
    mesh.map(to_bevy_mesh)
}

#[derive(Default, Eq, PartialEq, Clone, Copy)]
pub struct ChunkPriority {
    pub priority: usize,
}

impl PartialOrd for ChunkPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChunkPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.partial_cmp(&other.priority).unwrap()
    }
}

#[derive(Default, Resource)]
pub struct LoadQueue {
    inner: DefaultMapIPQ<ChunkPosition, ChunkPriority>,
}

impl LoadQueue {
    pub fn push(&mut self, pos: ChunkPosition, priority: ChunkPriority) {
        self.inner.push(pos, priority);
    }

    pub fn remove(&mut self, pos: ChunkPosition) {
        if self.inner.contains(pos) {
            self.inner.remove(pos);
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn pop(&mut self) -> Option<ChunkPosition> {
        self.inner.pop()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
