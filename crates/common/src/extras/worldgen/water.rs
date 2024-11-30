use crate::chunks::{Chunk, CHUNK_SIZE};
use crate::extras::block_ids::{GRASS_BLOCK_ID, WATER_BLOCK_ID};
use crate::extras::chunks;
use crate::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::zoom::ZoomLevel;
use rustc_hash::FxHashMap;

/// A flat water world with solid blocks in the corners of chunks.
#[derive(Debug, Default)]
pub struct Water {
    block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Water {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&mut self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
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
