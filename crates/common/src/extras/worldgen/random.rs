use rand::Rng;
use rustc_hash::FxHashMap;

use crate::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE};
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use crate::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::zoom::ZoomLevel;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Random {
    pub block_mappings: FxHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Random {
    fn initialize(&mut self, mappings: FxHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
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
        let mut rng = rand::thread_rng();
        let mut chunk = UnpackedChunk::default();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height: i8 = rng.gen_range(0..=2);
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
