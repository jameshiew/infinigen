use ahash::AHashMap;

use crate::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE};
use crate::extras::block_ids::GRASS_BLOCK_ID;
use crate::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::zoom::ZoomLevel;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SingleBlock {
    pub block_mappings: AHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for SingleBlock {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&self, _pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom?
        let mut chunk = UnpackedChunk::default();
        chunk.insert(
            &BlockPosition {
                x: CHUNK_SIZE / 2,
                y: 0,
                z: CHUNK_SIZE / 2,
            },
            *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
        );
        chunk.into()
    }
}
