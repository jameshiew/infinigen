use block_mesh::ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
use linearize::{StaticCopyMap, static_copy_map};

use crate::chunks::CHUNK_SIZE_U32;
use crate::mesh::block::VoxelBlock;
use crate::world::Direction;

// 1-voxel boundary padding around the chunk is necessary
pub const PADDED_CHUNK_SIZE: u32 = CHUNK_SIZE_U32 + 2;
pub const PADDED_CHUNK_MAX_INDEX: u32 = PADDED_CHUNK_SIZE - 1;

pub type PaddedChunkShape = ConstShape3u32<PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE>;
pub type PaddedChunk = [VoxelBlock; PaddedChunkShape::USIZE];

/// Represents a face of a chunk - when meshing a chunk, we need the faces of the neighboring chunks.
pub type ChunkFaceShape = ConstShape2u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
pub type ChunkFace = [VoxelBlock; ChunkFaceShape::USIZE];

pub const EMPTY_CHUNK_FACE: ChunkFace = [VoxelBlock::Empty; ChunkFaceShape::USIZE];
pub const EMPTY_CHUNK_FACES: StaticCopyMap<Direction, ChunkFace> = static_copy_map! {
    of type Direction:
    _ => EMPTY_CHUNK_FACE,
};
