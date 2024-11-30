use infinigen_common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_I32};
use infinigen_common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use crate::extras::block_ids::{DIRT_BLOCK_ID, GRASS_BLOCK_ID, STONE_BLOCK_ID};
use crate::extras::chunks;
use rustc_hash::FxHashMap;

/// Similar to Flat, but with a 1-block high border around each block, and a x+z tower of blocks in the middle. Chunks above the ground chunk have a block centred in the middle.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BorderedTowers {
    pub block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for BorderedTowers {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&mut self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom
        if pos.y < -1 {
            return Chunk::Empty;
        }
        if pos.y == -1 {
            return chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into();
        }
        let mut chunk = UnpackedChunk::default();
        if pos.y == 0 {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if x == 0 || x == CHUNK_SIZE - 1 || z == 0 || z == CHUNK_SIZE - 1 {
                        // border
                        chunk.insert(
                            &BlockPosition { x, y: 0, z },
                            *self.block_mappings.get(STONE_BLOCK_ID).unwrap(),
                        );
                    } else if x == CHUNK_SIZE / 2 && z == CHUNK_SIZE / 2 {
                        // tower
                        for y in 0..(pos.x.abs() + pos.z.abs()).min(CHUNK_SIZE_I32 - 1) {
                            chunk.insert(
                                &BlockPosition { x, y: y as i8, z },
                                *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                            );
                        }
                    }
                }
            }
            return chunk.into();
        }
        chunk.insert(
            &BlockPosition {
                x: CHUNK_SIZE / 2,
                y: 0,
                z: CHUNK_SIZE / 2,
            },
            *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
        );
        chunk.into()
    }
}
