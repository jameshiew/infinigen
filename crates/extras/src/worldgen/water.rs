use ahash::AHashMap;
use infinigen_common::chunks::{Chunk, CHUNK_SIZE};
use infinigen_common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::block_ids::{GRASS_BLOCK_ID, WATER_BLOCK_ID};

/// A flat water world with solid blocks in the corners of chunks.
#[derive(Debug, Default)]
pub struct Water {
    block_mappings: AHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Water {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // zoom doesn't change anything?
        if pos.y == -1 {
            let mut chunk = infinigen_common::chunks::filled_chunk(
                *self.block_mappings.get(WATER_BLOCK_ID).unwrap(),
            );
            for x in [0, CHUNK_SIZE - 1] {
                for z in [0, CHUNK_SIZE - 1] {
                    chunk.insert(
                        &BlockPosition {
                            x,
                            y: CHUNK_SIZE - 1,
                            z,
                        },
                        *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                    );
                }
            }
            chunk.into()
        } else {
            Chunk::Empty
        }
    }
}
