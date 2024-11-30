//! Helpers for creating chunks.
use infinigen_common::chunks::{UnpackedChunk, CHUNK_SIZE};
use infinigen_common::world::{BlockPosition, ChunkBlockId};

pub fn filled_chunk(block: ChunkBlockId) -> UnpackedChunk {
    let mut chunk = UnpackedChunk::default();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.insert(&BlockPosition { x, y, z }, block);
            }
        }
    }
    chunk
}

/// Chunk where the topmost layer is dirt - can be used to represent the ground.
pub fn top_chunk(block: ChunkBlockId) -> UnpackedChunk {
    let mut chunk = UnpackedChunk::default();
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            chunk.insert(
                &BlockPosition {
                    x,
                    y: CHUNK_SIZE - 1,
                    z,
                },
                block,
            );
        }
    }
    chunk
}
