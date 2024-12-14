use ahash::AHashMap;
use infinigen_common::blocks::BlockID;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::blocks::DIRT_BLOCK_ID;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Flat {
    pub block_mappings: AHashMap<BlockID, MappedBlockID>,
}

impl WorldGen for Flat {
    fn initialize(&mut self, mappings: AHashMap<BlockID, MappedBlockID>) {
        self.block_mappings = mappings;
    }

    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // zoom doesn't change anything
        if pos.y == -1 {
            infinigen_common::chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap())
                .into()
        } else {
            Chunk::Empty
        }
    }
}
