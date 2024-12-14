use ahash::AHashMap;
use infinigen_common::blocks::BlockID;
use infinigen_common::chunks::{Array3Chunk, Chunk, CHUNK_SIZE};
use infinigen_common::world::{BlockPosition, ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::blocks::GRASS_BLOCK_ID;

/// Generates a single block in the middle of every chunk.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SingleBlock {
    grass: MappedBlockID,
}

impl WorldGen for SingleBlock {
    fn initialize(&mut self, mappings: AHashMap<BlockID, MappedBlockID>) {
        self.grass = *mappings.get(GRASS_BLOCK_ID).unwrap();
    }

    fn get(&self, _pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom?
        let mut chunk = Array3Chunk::default();
        chunk.insert(
            &BlockPosition {
                x: CHUNK_SIZE / 2,
                y: 0,
                z: CHUNK_SIZE / 2,
            },
            self.grass,
        );
        chunk.into()
    }
}
