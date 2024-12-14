use ahash::AHashMap;
use infinigen_common::blocks::BlockID;
use infinigen_common::chunks::Chunk;
use infinigen_common::world::{ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::blocks::DIRT_BLOCK_ID;

/// Generates a completely flat world of dirt at y=0.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Flat {
    dirt: MappedBlockID,
}

impl WorldGen for Flat {
    fn initialize(&mut self, mappings: AHashMap<BlockID, MappedBlockID>) {
        self.dirt = *mappings.get(DIRT_BLOCK_ID).unwrap();
    }

    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // zoom level does not change anything
        if pos.y == -1 {
            infinigen_common::chunks::top_chunk(self.dirt).into()
        } else {
            Chunk::Empty
        }
    }
}
