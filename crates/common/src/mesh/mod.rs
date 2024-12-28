//! This module is concerned with meshing chunks, and not with rendering.
use block_mesh::ndshape::ConstShape;
use block_mesh::{
    greedy_quads, visible_block_faces, GreedyQuadsBuffer, UnitQuadBuffer, UnorientedQuad, Voxel,
    VoxelVisibility,
};
use faces::RHS_FACES;
use shapes::{PaddedChunk, PaddedChunkShape, PADDED_CHUNK_MAX_INDEX};

use self::block::VoxelBlock;
use self::textures::{BlockAppearances, Face};

pub mod block;
pub mod faces;
pub mod shapes;
pub mod textures;

/// Stores details of a mesh, to be passed to a GPU for rendering.
pub struct MeshInfo {
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

/// Returns a mesh of all visible block faces in the chunk.
/// adapted from <https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs>. Returns `None` if there are no visible faces.
pub fn mesh_chunk_visible_block_faces(
    padded: &PaddedChunk,
    block_textures: &BlockAppearances,
) -> Option<MeshInfo> {
    let mut buffer = UnitQuadBuffer::new();
    visible_block_faces(
        padded,
        &PaddedChunkShape {},
        [0; 3],
        [PADDED_CHUNK_MAX_INDEX; 3],
        &RHS_FACES,
        &mut buffer,
    );
    if buffer.num_quads() == 0 {
        return None;
    }

    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut colors = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.groups.into_iter().zip(RHS_FACES.into_iter()) {
        for quad in group.into_iter() {
            let uq: UnorientedQuad = quad.into();
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            let pstns = face.quad_mesh_positions(&uq, 1.0);
            positions.extend_from_slice(&pstns);
            let nrmls = face.quad_mesh_normals();
            normals.extend_from_slice(&nrmls);

            // all normals should be identical so we can just take the first one (it will be one of the six cardinal directions - up, down, north, east, south, west)
            let normal = nrmls[0];
            // front face on south (+Z) by default
            let mut block_face = match normal {
                [0., 0., 1.] => Face::Back,
                [0., 0., -1.] => Face::Front,
                [0., 1., 0.] => Face::Top,
                [0., -1., 0.] => Face::Bottom,
                [1., 0., 0.] => Face::Right,
                [-1., 0., 0.] => Face::Left,
                _ => panic!("unexpected value for normal"),
            };

            let mut padded_chunk_coord = [
                pstns[0][0].floor() as u32,
                pstns[0][1].floor() as u32,
                pstns[0][2].floor() as u32,
            ];
            if face.n_sign() == 1 {
                padded_chunk_coord[0] -= normal[0] as u32;
                padded_chunk_coord[1] -= normal[1] as u32;
                padded_chunk_coord[2] -= normal[2] as u32;
            }
            let mut flip_v = false;
            match block_face {
                Face::Top | Face::Bottom => {}
                Face::Left | Face::Right | Face::Front | Face::Back => {
                    flip_v = true;

                    // temporary hack to use bottom texture instead of side texture if there is an opaque block on top
                    // only really makes sense for e.g. grass/snow blocks which should use a their bottom dirt texture on the side if occluded on top
                    let above_coord = PaddedChunkShape::linearize([
                        padded_chunk_coord[0],
                        padded_chunk_coord[1] + 1,
                        padded_chunk_coord[2],
                    ]);
                    let above = padded[above_coord as usize];
                    if above.get_visibility() == VoxelVisibility::Opaque {
                        block_face = Face::Bottom;
                    }
                }
            }

            let i = PaddedChunkShape::linearize(padded_chunk_coord);
            let block = padded[i as usize];

            let texture_uvs = face.tex_coords(face.permutation().axes()[0], flip_v, &uq);
            let VoxelBlock::Opaque(chunk_block_id) = block else {
                unimplemented!("only opaque blocks are supported")
            };

            match block_textures.get(&chunk_block_id, block_face, texture_uvs) {
                Some(appearance) => match appearance {
                    textures::FaceAppearanceTransformed::Texture { coords } => {
                        uvs.extend_from_slice(&coords);
                        colors.extend_from_slice(&[[1.; 4]; 4]);
                    }
                    textures::FaceAppearanceTransformed::Color { r, g, b, a } => {
                        uvs.extend_from_slice(&[[0.; 2]; 4]);
                        colors.extend_from_slice(&[[r, g, b, a]; 4]);
                    }
                },
                None => {
                    tracing::error!(
                        ?chunk_block_id,
                        ?block_face,
                        "No appearance defined for block face"
                    )
                }
            }
        }
    }
    Some(MeshInfo {
        indices,
        positions,
        normals,
        colors,
        uvs,
    })
}

/// Returns a mesh of quads in a chunk.
/// adapted from <https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs>
pub fn mesh_chunk_greedy_quads(
    padded: &PaddedChunk,
    block_textures: &BlockAppearances,
) -> Option<MeshInfo> {
    let mut buffer = GreedyQuadsBuffer::new(padded.len());
    greedy_quads(
        padded,
        &PaddedChunkShape {},
        [0; 3],
        [PADDED_CHUNK_MAX_INDEX; 3],
        &RHS_FACES,
        &mut buffer,
    );
    if buffer.quads.num_quads() == 0 {
        return None;
    }

    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut colors = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(RHS_FACES.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            let pstns = face.quad_mesh_positions(&quad, 1.0);
            positions.extend_from_slice(&pstns);
            let nrmls = face.quad_mesh_normals();
            normals.extend_from_slice(&nrmls);
            uvs.extend_from_slice(&face.tex_coords(face.permutation().axes()[0], false, &quad));

            // all normals should be identical so we can just take the first one (it will be one of the six cardinal directions - up, down, north, east, south, west)
            // TODO: probably some of this logic isn't important when rendering water
            let normal = nrmls[0];
            // front face on south (+Z) by default
            let mut block_face = match normal {
                [0., 0., 1.] => Face::Back,
                [0., 0., -1.] => Face::Front,
                [0., 1., 0.] => Face::Top,
                [0., -1., 0.] => Face::Bottom,
                [1., 0., 0.] => Face::Right,
                [-1., 0., 0.] => Face::Left,
                _ => panic!("unexpected value for normal"),
            };

            let mut padded_chunk_coord = [
                pstns[0][0].floor() as u32,
                pstns[0][1].floor() as u32,
                pstns[0][2].floor() as u32,
            ];
            if face.n_sign() == 1 {
                padded_chunk_coord[0] -= normal[0] as u32;
                padded_chunk_coord[1] -= normal[1] as u32;
                padded_chunk_coord[2] -= normal[2] as u32;
            }

            let i = PaddedChunkShape::linearize(padded_chunk_coord);
            let block = padded[i as usize];
            let VoxelBlock::Translucent(chunk_block_id) = block else {
                unimplemented!("only translucent blocks are supported")
            };
            match block_face {
                Face::Top | Face::Bottom => {}
                Face::Left | Face::Right | Face::Front | Face::Back => {
                    // temporary hack to use bottom texture instead of side texture if there is an opaque block on top
                    // only really makes sense for e.g. grass/snow blocks which should use a their bottom dirt texture on the side if occluded on top
                    let above_coord = PaddedChunkShape::linearize([
                        padded_chunk_coord[0],
                        padded_chunk_coord[1] + 1,
                        padded_chunk_coord[2],
                    ]);
                    let above = padded[above_coord as usize];
                    if above.get_visibility() == VoxelVisibility::Opaque {
                        block_face = Face::Bottom;
                    }
                }
            }

            let mut color = [0., 0., 0., 1.];
            if let Some(actual_color) = block_textures.to_color(&chunk_block_id, block_face) {
                color = actual_color;
            }
            colors.extend_from_slice(&[color; 4]);
        }
    }
    Some(MeshInfo {
        indices,
        positions,
        normals,
        colors,
        uvs,
    })
}
