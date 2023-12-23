use crate::common::chunks::Chunk;
use crate::common::world::{BlockId, ChunkBlockId, ChunkPosition, WorldGen};
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Flat {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Flat {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&mut self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // zoom doesn't change anything
        if pos.y == -1 {
            chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
        } else {
            Chunk::Empty
        }
    }
}
