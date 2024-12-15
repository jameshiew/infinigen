use block_mesh::ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};

use crate::chunks::CHUNK_SIZE_U32;
use crate::mesh::block::VoxelBlock;

// 1-voxel boundary padding around the chunk is necessary
pub const PADDED_CHUNK_SIZE: u32 = CHUNK_SIZE_U32 + 2;
pub const PADDED_CHUNK_MAX_INDEX: u32 = PADDED_CHUNK_SIZE - 1;

pub type PaddedChunkShape = ConstShape3u32<PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE>;
pub type PaddedChunk = [VoxelBlock; PaddedChunkShape::SIZE as usize];

/// Represents a face of a chunk - when meshing a chunk, we need the faces of the neighboring chunks.
pub type ChunkFaceShape = ConstShape2u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
pub type ChunkFace = [VoxelBlock; ChunkFaceShape::SIZE as usize];

pub const EMPTY_CHUNK_FACE: ChunkFace = [VoxelBlock::Empty; ChunkFaceShape::SIZE as usize];
pub const EMPTY_CHUNK_FACES: [ChunkFace; 6] = [EMPTY_CHUNK_FACE; 6];
