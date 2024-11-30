use infinigen_common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE};
use infinigen_common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use crate::extras::block_ids::GRASS_BLOCK_ID;
use rustc_hash::FxHashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SingleBlock {
    pub block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for SingleBlock {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&mut self, _pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
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
