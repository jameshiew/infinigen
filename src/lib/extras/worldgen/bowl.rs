use crate::common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE};
use crate::common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::common::zoom::ZoomLevel;
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use rustc_hash::FxHashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bowl {
    pub block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Bowl {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&mut self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom
        if pos.y < -1 {
            return Chunk::Empty;
        }
        if pos.y == -1 {
            return chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into();
        }
        if pos.y >= 1 {
            return Chunk::Empty;
        }

        let mut chunk = UnpackedChunk::default();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = ((pos.x.abs() + pos.z.abs()) as i8).min(CHUNK_SIZE - 1);
                for y in 0..height {
                    chunk.insert(
                        &BlockPosition { x, y, z },
                        *self.block_mappings.get(DIRT_BLOCK_ID).unwrap(),
                    );
                }
            }
        }
        chunk.into()
    }
}
