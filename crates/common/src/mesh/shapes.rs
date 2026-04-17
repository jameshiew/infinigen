use linearize::{StaticCopyMap, static_copy_map};

use crate::chunks::CHUNK_SIZE_U32;
use crate::mesh::block::VoxelBlock;
use crate::world::Direction;

/// 1-voxel boundary padding around the chunk is required so that meshing
/// can look at neighbors without bounds checks.
pub const PADDED_CHUNK_SIZE: u32 = CHUNK_SIZE_U32 + 2;
pub const PADDED_CHUNK_MAX_INDEX: u32 = PADDED_CHUNK_SIZE - 1;
pub const PADDED_CHUNK_VOLUME: usize =
    (PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE * PADDED_CHUNK_SIZE) as usize;

pub type PaddedChunk = [VoxelBlock; PADDED_CHUNK_VOLUME];

/// Linearize a 3D padded-chunk coordinate to an index with X innermost.
#[inline]
pub const fn padded_linearize([x, y, z]: [u32; 3]) -> usize {
    (x + PADDED_CHUNK_SIZE * (y + PADDED_CHUNK_SIZE * z)) as usize
}

pub const CHUNK_FACE_VOLUME: usize = (CHUNK_SIZE_U32 * CHUNK_SIZE_U32) as usize;

/// Represents a face of a chunk — used to expose neighbor chunks' boundary
/// voxels during meshing.
pub type ChunkFace = [VoxelBlock; CHUNK_FACE_VOLUME];

#[inline]
pub const fn chunk_face_linearize([u, v]: [u32; 2]) -> usize {
    (u + CHUNK_SIZE_U32 * v) as usize
}

pub const EMPTY_CHUNK_FACE: ChunkFace = [VoxelBlock::Empty; CHUNK_FACE_VOLUME];
pub const EMPTY_CHUNK_FACES: StaticCopyMap<Direction, ChunkFace> = static_copy_map! {
    of type Direction:
    _ => EMPTY_CHUNK_FACE,
};
