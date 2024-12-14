use ahash::AHashMap;
use infinigen_common::blocks::BlockId;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::blocks::DIRT_BLOCK_ID;

#[derive(Debug, Default, Clone)]
pub struct Alternating {
    block_mappings: AHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Alternating {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom?
        if pos.y % 2 == 0 {
            if pos.x % 2 == 0 || pos.z % 2 == 0 {
                Chunk::Empty
            } else {
                infinigen_common::chunks::filled_chunk(
                    *self.block_mappings.get(DIRT_BLOCK_ID).unwrap(),
                )
                .into()
            }
        } else if pos.x % 2 == 0 || pos.z % 2 == 0 {
            infinigen_common::chunks::filled_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap())
                .into()
        } else {
            Chunk::Empty
        }
    }
}
