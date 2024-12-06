//! Converts from our native mesh types to Bevy meshes

use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use infinigen_common::chunks::UnpackedChunk;
use infinigen_common::mesh::faces::prepare_padded_chunk;
use infinigen_common::mesh::shapes::ChunkFace;
use infinigen_common::mesh::{
    mesh_chunk_greedy_quads, mesh_chunk_visible_block_faces, textures::TextureMap, MeshInfo,
};

use crate::assets::blocks::BlockMappings;

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
    chunk: &UnpackedChunk,
    neighbor_faces: &[ChunkFace; 6],
    block_textures: &TextureMap,
    block_mappings: &BlockMappings,
) -> Option<Mesh> {
    let samples = prepare_padded_chunk(chunk, neighbor_faces, block_mappings);
    let mesh = mesh_chunk_visible_block_faces(&samples, block_textures);
    mesh.map(to_bevy_mesh)
}

/// Returns a mesh of quads in a chunk.
/// adapted from <https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs>
pub fn bevy_mesh_greedy_quads(
    chunk: &UnpackedChunk,
    neighbor_faces: &[ChunkFace; 6],
    block_textures: &TextureMap,
    block_mappings: &BlockMappings,
) -> Option<Mesh> {
    let samples = prepare_padded_chunk(chunk, neighbor_faces, block_mappings);
    let mesh = mesh_chunk_greedy_quads(&samples, block_textures);
    mesh.map(to_bevy_mesh)
}
