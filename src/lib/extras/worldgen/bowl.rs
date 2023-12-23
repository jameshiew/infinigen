use crate::common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE};
use crate::common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::extras::block_ids::DIRT_BLOCK_ID;
use crate::extras::chunks;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bowl {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Bowl {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&mut self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
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
