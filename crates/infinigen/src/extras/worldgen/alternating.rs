use infinigen_common::chunks::Chunk;
use infinigen_common::world::{BlockId, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use rustc_hash::FxHashMap;

#[derive(Debug, Default, Clone)]
pub struct Alternating {
    block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Alternating {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&mut self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
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
