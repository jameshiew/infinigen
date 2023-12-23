use crate::common::chunks::Chunk;
use crate::common::world::{BlockId, ChunkBlockId, ChunkPosition, WorldGen};
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Alternating {
    block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Alternating {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // TODO: implement zoom?
        if pos.y % 2 == 0 {
            if pos.x % 2 == 0 || pos.z % 2 == 0 {
                Chunk::Empty
            } else {
                chunks::filled_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
            }
        } else if pos.x % 2 == 0 || pos.z % 2 == 0 {
            chunks::filled_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
        } else {
            Chunk::Empty
        }
    }
}
