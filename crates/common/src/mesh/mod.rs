//! Chunk meshing — converts padded chunks of voxels into triangle-list meshes
//! ready for upload to the GPU.
//!
//! Two meshing strategies are provided:
//!
//! - [`mesh_chunk_visible_block_faces`] emits one unit quad per visible face,
//!   used for opaque geometry.
//! - [`mesh_chunk_greedy_quads`] merges adjacent face quads into larger
//!   rectangles, used for translucent geometry (water, etc).

use self::block::{VoxelBlock, VoxelVisibility};
use self::quad::{FaceDir, Quad};
use self::shapes::{PADDED_CHUNK_MAX_INDEX, PaddedChunk, padded_linearize};
use self::textures::BlockAppearances;
use crate::blocks::Face;

pub mod block;
pub mod faces;
pub mod quad;
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

/// Whether the face between `voxel` (owner) and `neighbor` should be meshed.
///
/// Mirrors the rule used by `block-mesh-rs` so that visible geometry matches
/// the old behavior: opaque faces are emitted next to anything non-opaque,
/// translucent faces are emitted only next to empty voxels.
#[inline]
const fn face_needs_mesh(voxel: VoxelVisibility, neighbor: VoxelVisibility) -> bool {
    match (voxel, neighbor) {
        (VoxelVisibility::Empty, _) | (_, VoxelVisibility::Opaque) => false,
        (VoxelVisibility::Opaque, _) => true,
        (VoxelVisibility::Translucent, VoxelVisibility::Empty) => true,
        (VoxelVisibility::Translucent, VoxelVisibility::Translucent) => false,
    }
}

/// Emits one unit quad per visible block face in the padded chunk.
///
/// The returned array is indexed by [`FaceDir`] discriminant.
pub fn visible_block_faces_quads(padded: &PaddedChunk) -> [Vec<Quad>; 6] {
    let mut out: [Vec<Quad>; 6] = Default::default();

    // Iterate interior voxels only. The 1-voxel padding lets us always look
    // at neighbors without bounds checks.
    for z in 1..PADDED_CHUNK_MAX_INDEX {
        for y in 1..PADDED_CHUNK_MAX_INDEX {
            for x in 1..PADDED_CHUNK_MAX_INDEX {
                let voxel = padded[padded_linearize([x, y, z])];
                let vis = voxel.visibility();
                if matches!(vis, VoxelVisibility::Empty) {
                    continue;
                }
                for &face in &FaceDir::ALL {
                    let [nx, ny, nz] = face.normal();
                    let neighbor_idx = padded_linearize([
                        (x as i32 + nx) as u32,
                        (y as i32 + ny) as u32,
                        (z as i32 + nz) as u32,
                    ]);
                    let neighbor = padded[neighbor_idx];
                    if face_needs_mesh(vis, neighbor.visibility()) {
                        out[face as usize].push(Quad {
                            voxel: [x, y, z],
                            width: 1,
                            height: 1,
                        });
                    }
                }
            }
        }
    }
    out
}

/// Greedy-mesh the padded chunk, merging adjacent visible faces of the same
/// voxel into larger quads to reduce triangle count.
///
/// The returned array is indexed by [`FaceDir`] discriminant.
pub fn greedy_quads(padded: &PaddedChunk) -> [Vec<Quad>; 6] {
    let mut out: [Vec<Quad>; 6] = Default::default();
    for &face in &FaceDir::ALL {
        greedy_quads_for_face(padded, face, &mut out[face as usize]);
    }
    out
}

fn greedy_quads_for_face(padded: &PaddedChunk, face: FaceDir, out: &mut Vec<Quad>) {
    let normal_axis = face.normal_axis();
    let u_axis = face.u_axis();
    let v_axis = face.v_axis();
    let normal = face.normal();

    let min: u32 = 1;
    let max: u32 = PADDED_CHUNK_MAX_INDEX; // exclusive
    let u_size = max - min;
    let v_size = max - min;

    let mut mask: Vec<Option<VoxelBlock>> = vec![None; (u_size * v_size) as usize];

    for n_coord in min..max {
        // Populate the mask.
        for vi in 0..v_size {
            for ui in 0..u_size {
                let coord =
                    coord_from_axes(normal_axis, u_axis, v_axis, n_coord, min + ui, min + vi);
                let voxel = padded[padded_linearize(coord)];
                let vis = voxel.visibility();
                if matches!(vis, VoxelVisibility::Empty) {
                    mask[(ui + vi * u_size) as usize] = None;
                    continue;
                }
                let neighbor_idx = padded_linearize([
                    (coord[0] as i32 + normal[0]) as u32,
                    (coord[1] as i32 + normal[1]) as u32,
                    (coord[2] as i32 + normal[2]) as u32,
                ]);
                let neighbor = padded[neighbor_idx];
                mask[(ui + vi * u_size) as usize] = if face_needs_mesh(vis, neighbor.visibility()) {
                    Some(voxel)
                } else {
                    None
                };
            }
        }

        // Scan the mask and emit greedy quads.
        let mut vi = 0u32;
        while vi < v_size {
            let mut ui = 0u32;
            while ui < u_size {
                let Some(current) = mask[(ui + vi * u_size) as usize] else {
                    ui += 1;
                    continue;
                };

                // Extend width along u.
                let mut w = 1u32;
                while ui + w < u_size && mask[((ui + w) + vi * u_size) as usize] == Some(current) {
                    w += 1;
                }

                // Extend height along v: each candidate row must match the
                // full current width.
                let mut h = 1u32;
                'outer: while vi + h < v_size {
                    for k in 0..w {
                        if mask[((ui + k) + (vi + h) * u_size) as usize] != Some(current) {
                            break 'outer;
                        }
                    }
                    h += 1;
                }

                let voxel =
                    coord_from_axes(normal_axis, u_axis, v_axis, n_coord, min + ui, min + vi);
                out.push(Quad {
                    voxel,
                    width: w,
                    height: h,
                });

                // Mark the covered cells as consumed.
                for jj in 0..h {
                    for ii in 0..w {
                        mask[((ui + ii) + (vi + jj) * u_size) as usize] = None;
                    }
                }

                ui += w;
            }
            vi += 1;
        }
    }
}

#[inline]
const fn coord_from_axes(
    normal_axis: usize,
    u_axis: usize,
    v_axis: usize,
    n: u32,
    u: u32,
    v: u32,
) -> [u32; 3] {
    let mut coord = [0u32; 3];
    coord[normal_axis] = n;
    coord[u_axis] = u;
    coord[v_axis] = v;
    coord
}

/// Build a mesh of all visible opaque block faces in the padded chunk.
pub fn mesh_chunk_visible_block_faces(
    padded: &PaddedChunk,
    block_textures: &BlockAppearances,
) -> Option<MeshInfo> {
    let quads = visible_block_faces_quads(padded);
    build_mesh_info(padded, &quads, block_textures, MeshStyle::Opaque)
}

/// Build a greedy-meshed translucent mesh from the padded chunk.
pub fn mesh_chunk_greedy_quads(
    padded: &PaddedChunk,
    block_textures: &BlockAppearances,
) -> Option<MeshInfo> {
    let quads = greedy_quads(padded);
    build_mesh_info(padded, &quads, block_textures, MeshStyle::Translucent)
}

enum MeshStyle {
    Opaque,
    Translucent,
}

fn build_mesh_info(
    padded: &PaddedChunk,
    quads: &[Vec<Quad>; 6],
    block_textures: &BlockAppearances,
    style: MeshStyle,
) -> Option<MeshInfo> {
    let total_quads: usize = quads.iter().map(Vec::len).sum();
    if total_quads == 0 {
        return None;
    }

    let mut indices = Vec::with_capacity(total_quads * 6);
    let mut positions = Vec::with_capacity(total_quads * 4);
    let mut normals = Vec::with_capacity(total_quads * 4);
    let mut colors = Vec::with_capacity(total_quads * 4);
    let mut uvs = Vec::with_capacity(total_quads * 4);

    for (face_idx, face_quads) in quads.iter().enumerate() {
        let face = FaceDir::ALL[face_idx];
        let normal = face.normal_f32();
        let is_side = matches!(
            face,
            FaceDir::XNeg | FaceDir::XPos | FaceDir::ZNeg | FaceDir::ZPos
        );

        for quad in face_quads {
            let base = positions.len() as u32;
            indices.extend_from_slice(&Quad::indices(base, face));
            positions.extend_from_slice(&quad.positions(face));
            normals.extend_from_slice(&[normal; 4]);

            let voxel = padded[padded_linearize(quad.voxel)];

            // Texture-fallback hack: grass/snow-style blocks should use their
            // bottom texture on side faces when there's an opaque block
            // sitting on top (the side texture would look wrong without
            // visible grass capping it).
            let mut block_face = face.block_face();
            if is_side {
                let above_idx = padded_linearize([quad.voxel[0], quad.voxel[1] + 1, quad.voxel[2]]);
                if matches!(padded[above_idx].visibility(), VoxelVisibility::Opaque) {
                    block_face = Face::Bottom;
                }
            }

            match style {
                MeshStyle::Opaque => {
                    let VoxelBlock::Opaque(chunk_block_id) = voxel else {
                        unimplemented!("only opaque blocks are supported")
                    };
                    let flip_v = is_side;
                    let face_uvs = quad.uvs(face, flip_v);
                    match block_textures.get(&chunk_block_id, block_face, face_uvs) {
                        Some(textures::FaceAppearanceTransformed::Texture { coords }) => {
                            uvs.extend_from_slice(&coords);
                            colors.extend_from_slice(&[[1.0; 4]; 4]);
                        }
                        Some(textures::FaceAppearanceTransformed::Color { r, g, b, a }) => {
                            uvs.extend_from_slice(&[[0.0; 2]; 4]);
                            colors.extend_from_slice(&[[r, g, b, a]; 4]);
                        }
                        None => {
                            tracing::error!(
                                ?chunk_block_id,
                                ?block_face,
                                "No appearance defined for block face"
                            );
                            uvs.extend_from_slice(&[[0.0; 2]; 4]);
                            colors.extend_from_slice(&[[0.0; 4]; 4]);
                        }
                    }
                }
                MeshStyle::Translucent => {
                    let VoxelBlock::Translucent(chunk_block_id) = voxel else {
                        unimplemented!("only translucent blocks are supported")
                    };
                    // Translucent blocks render as flat colors, so raw UVs
                    // are fine — no atlas lookup needed.
                    let face_uvs = quad.uvs(face, false);
                    uvs.extend_from_slice(&face_uvs);
                    let color = block_textures
                        .to_color(&chunk_block_id, block_face)
                        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
                    colors.extend_from_slice(&[color; 4]);
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
