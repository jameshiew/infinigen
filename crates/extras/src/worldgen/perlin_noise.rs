use ahash::AHashMap;
use infinigen_common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64};
use infinigen_common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use noise::{NoiseFn, Perlin};

use crate::block_ids::{DIRT_BLOCK_ID, GRASS_BLOCK_ID, STONE_BLOCK_ID, WATER_BLOCK_ID};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PerlinNoise {
    pub block_mappings: AHashMap<BlockId, ChunkBlockId>,
    seed: u32,
}

/// Must be smaller than [`CHUNK_SIZE_F64`]
const MAX_HEIGHT: f64 = CHUNK_SIZE_F64 / 2.;
const MAX_DEPTH: f64 = CHUNK_SIZE_F64 / 4.;

impl WorldGen for PerlinNoise {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Chunk {
        // TODO: implement zoom
        if pos.y < -1 {
            return Chunk::Empty;
        }
        if pos.y >= 1 {
            return Chunk::Empty;
        }

        let perlin = Perlin::new(self.seed);
        let chunk = if pos.y == -1 {
            let mut chunk = infinigen_common::chunks::filled_chunk(
                *self.block_mappings.get(DIRT_BLOCK_ID).unwrap(),
            );
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    chunk.insert(
                        &BlockPosition {
                            x,
                            y: CHUNK_SIZE - 1,
                            z,
                        },
                        *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                    );

                    let val = perlin.get([
                        x as f64 / CHUNK_SIZE_F64 + pos.x as f64,
                        z as f64 / CHUNK_SIZE_F64 + pos.z as f64,
                    ]);

                    if val < -0.3 {
                        let val = val + 0.3;
                        let depth = (val.abs() * MAX_DEPTH).floor() as i8;
                        for y in 0..depth {
                            chunk.insert(
                                &BlockPosition {
                                    x,
                                    y: CHUNK_SIZE - 1 - y,
                                    z,
                                },
                                *self.block_mappings.get(WATER_BLOCK_ID).unwrap(),
                            );
                        }
                    }
                }
            }
            chunk
        } else {
            // i.e. pos.y == 0
            let mut chunk = UnpackedChunk::default();
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let val = perlin.get([
                        x as f64 / CHUNK_SIZE_F64 + pos.x as f64,
                        z as f64 / CHUNK_SIZE_F64 + pos.z as f64,
                    ]);

                    if val > 0. {
                        let height = (val * MAX_HEIGHT).floor() as i8;
                        for y in 0..height {
                            chunk.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(STONE_BLOCK_ID).unwrap(),
                            );
                        }
                    }
                }
            }
            chunk
        };
        chunk.into()
    }
}
