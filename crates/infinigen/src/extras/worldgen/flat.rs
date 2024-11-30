use infinigen_common::chunks::Chunk;
use infinigen_common::world::{BlockId, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use rustc_hash::FxHashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Flat {
    pub block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Flat {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&mut self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // zoom doesn't change anything
        if pos.y == -1 {
            chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
        } else {
            Chunk::Empty
        }
    }
}
