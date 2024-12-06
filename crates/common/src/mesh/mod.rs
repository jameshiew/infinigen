//! This module is concerned with meshing chunks, and not with rendering.
use block_mesh::{
    greedy_quads, ndshape::ConstShape, visible_block_faces, GreedyQuadsBuffer, UnitQuadBuffer,
};
use block_mesh::{UnorientedQuad, Voxel, VoxelVisibility};
use faces::RHS_FACES;
use rand::Rng;
use shapes::{PaddedChunk, PaddedChunkShape, PADDED_CHUNK_MAX_INDEX};

use self::block::VoxelBlock;
use self::textures::{Face, TextureMap};
use crate::chunks::{UnpackedChunk, CHUNK_SIZE};
use crate::world::{BlockPosition, LocalPosition};

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
    block_textures: &TextureMap,
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
    let mut prng = rand::thread_rng();
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

            // front face on south (+Z) by default
            let mut block_face = if normal == [0., 0., 1.] {
                Face::Back
            } else if normal == [0., 0., -1.] {
                Face::Front
            } else if normal == [0., 1., 0.] {
                Face::Top
            } else if normal == [0., -1., 0.] {
                Face::Bottom
            } else if normal == [1., 0., 0.] {
                Face::Right
            } else if normal == [-1., 0., 0.] {
                Face::Left
            } else {
                panic!("invalid normal")
            };
            let mut flip_v = false;
            match block_face {
                Face::Top => {
                    // temporary hack to break things up, rotate the top texture randomly
                    flip_v = prng.gen_bool(0.5);
                }
                Face::Bottom => {}
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
                panic!()
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
    block_textures: &TextureMap,
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
    let mut prng = rand::thread_rng();
    for (group, face) in buffer.quads.groups.into_iter().zip(RHS_FACES.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            let pstns = face.quad_mesh_positions(&quad, 1.0);
            positions.extend_from_slice(&pstns);
            let nrmls = face.quad_mesh_normals();
            normals.extend_from_slice(&nrmls);
            // all normals should be identical so we can just take the first one (it will be one of the six cardinal directions - up, down, north, east, south, west)
            // TODO: probably some of this logic isn't important when rendering water
            let normal = nrmls[0];
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
            uvs.extend_from_slice(&face.tex_coords(face.permutation().axes()[0], false, &quad));

            let i = PaddedChunkShape::linearize(padded_chunk_coord);
            let block = padded[i as usize];
            let VoxelBlock::Translucent(chunk_block_id) = block else {
                panic!()
            };

            // front face on south (+Z) by default
            let mut block_face = if normal == [0., 0., 1.] {
                Face::Back
            } else if normal == [0., 0., -1.] {
                Face::Front
            } else if normal == [0., 1., 0.] {
                Face::Top
            } else if normal == [0., -1., 0.] {
                Face::Bottom
            } else if normal == [1., 0., 0.] {
                Face::Right
            } else if normal == [-1., 0., 0.] {
                Face::Left
            } else {
                panic!("invalid normal")
            };
            let mut _flip_v = false;
            match block_face {
                Face::Top => {
                    // temporary hack to break things up, rotate the top texture randomly
                    _flip_v = prng.gen_bool(0.5);
                }
                Face::Bottom => {}
                Face::Left | Face::Right | Face::Front | Face::Back => {
                    _flip_v = true;

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

// Returns the local locations of all visible blocks in the chunk, which then need to be offset by the chunk position.
pub fn mesh_chunk_naive(chunk: UnpackedChunk) -> Vec<LocalPosition> {
    let mut positions = vec![];
    for bx in 0..CHUNK_SIZE {
        for by in 0..CHUNK_SIZE {
            for bz in 0..CHUNK_SIZE {
                let block = chunk.get(&BlockPosition {
                    x: bx,
                    y: by,
                    z: bz,
                });
                if block.is_none() {
                    continue;
                }

                let up = chunk.get(&BlockPosition {
                    x: bx,
                    y: (by + 1).min(CHUNK_SIZE - 1),
                    z: bz,
                });
                let down = chunk.get(&BlockPosition {
                    x: bx,
                    y: (by - 1).max(0),
                    z: bz,
                });
                let north = chunk.get(&BlockPosition {
                    x: bx,
                    y: by,
                    z: (bz - 1).max(0),
                });
                let east = chunk.get(&BlockPosition {
                    x: (bx + 1).min(CHUNK_SIZE - 1),
                    y: by,
                    z: bz,
                });
                let west = chunk.get(&BlockPosition {
                    x: (bx - 1).max(0),
                    y: by,
                    z: bz,
                });
                let south = chunk.get(&BlockPosition {
                    x: bx,
                    y: by,
                    z: (bz + 1).min(CHUNK_SIZE - 1),
                });
                if up.is_some()
                    && down.is_some()
                    && north.is_some()
                    && east.is_some()
                    && west.is_some()
                    && south.is_some()
                    && !(bx == 0
                        || bx == CHUNK_SIZE - 1
                        || by == 0
                        || by == CHUNK_SIZE - 1
                        || bz == 0
                        || bz == CHUNK_SIZE - 1)
                {
                    continue;
                }

                let (x, y, z) = (bx as f32, by as f32, bz as f32);
                positions.push(LocalPosition { x, y, z });
            }
        }
    }
    positions
}
