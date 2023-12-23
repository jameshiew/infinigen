use crate::common::chunks::{Chunk, CHUNK_SIZE};
use crate::common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::extras::block_ids::{GRASS_BLOCK_ID, WATER_BLOCK_ID};
use crate::extras::chunks;
use std::collections::HashMap;

/// A flat water world with solid blocks in the corners of chunks.
#[derive(Debug, Default)]
pub struct Water {
    block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Water {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&mut self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // zoom doesn't change anything?
        if pos.y == -1 {
            let mut chunk = chunks::filled_chunk(*self.block_mappings.get(WATER_BLOCK_ID).unwrap());
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
