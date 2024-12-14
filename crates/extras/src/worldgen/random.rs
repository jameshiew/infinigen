use ahash::AHashMap;
use infinigen_common::blocks::BlockId;
use infinigen_common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE};
use infinigen_common::world::{BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use rand::Rng;

use crate::blocks::DIRT_BLOCK_ID;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Random {
    pub block_mappings: AHashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Random {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom
        if pos.y < -1 {
            return Chunk::Empty;
        }
        if pos.y == -1 {
            return infinigen_common::chunks::top_chunk(
                *self.block_mappings.get(DIRT_BLOCK_ID).unwrap(),
            )
            .into();
        }
        if pos.y >= 1 {
            return Chunk::Empty;
        }
        let mut rng = rand::thread_rng();
        let mut chunk = UnpackedChunk::default();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = rng.gen_range(0..=2);
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
