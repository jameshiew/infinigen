use std::collections::HashMap;

use noise::{NoiseFn, Perlin};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_SIZE_I32};
use crate::common::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen};
use crate::extras::block_ids::{DIRT_BLOCK_ID, GRASS_BLOCK_ID, STONE_BLOCK_ID, WATER_BLOCK_ID};
use crate::extras::chunks;

pub mod experiment1;
pub mod mountain_archipelago;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldGenTypes {
    Flat,
    BorderedTowers,
    Random,
    PerlinNoise,
    Water,
    MountainIslands,
    Alternating,
    SingleBlock,
    Experiment1,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SingleBlock {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for SingleBlock {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&self, _pos: &ChunkPosition, _zoom: f64) -> Chunk {
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Flat {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Flat {
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // zoom doesn't change anything
        if pos.y == -1 {
            chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
        } else {
            Chunk::Empty
        }
    }

    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
}

/// Similar to [`Flat`], but with a 1-block high border around each block, and a x+z tower of blocks in the middle. Chunks above the ground chunk have a block centred in the middle.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BorderedTowers {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for BorderedTowers {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // TODO: implement zoom
        if pos.y < -1 {
            return Chunk::Empty;
        }
        if pos.y == -1 {
            return chunks::top_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into();
        }
        let mut chunk = UnpackedChunk::default();
        if pos.y == 0 {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if x == 0 || x == CHUNK_SIZE - 1 || z == 0 || z == CHUNK_SIZE - 1 {
                        // border
                        chunk.insert(
                            &BlockPosition { x, y: 0, z },
                            *self.block_mappings.get(STONE_BLOCK_ID).unwrap(),
                        );
                    } else if x == CHUNK_SIZE / 2 && z == CHUNK_SIZE / 2 {
                        // tower
                        for y in 0..(pos.x.abs() + pos.z.abs()).min(CHUNK_SIZE_I32 - 1) {
                            chunk.insert(
                                &BlockPosition { x, y: y as i8, z },
                                *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                            );
                        }
                    }
                }
            }
            return chunk.into();
        }
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Random {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Random {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bowl {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Bowl {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PerlinNoise {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
    seed: u32,
}

/// Must be smaller than [`CHUNK_SIZE_F64`]
const MAX_HEIGHT: f64 = CHUNK_SIZE_F64 / 2.;
const MAX_DEPTH: f64 = CHUNK_SIZE_F64 / 4.;

impl WorldGen for PerlinNoise {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // TODO: implement zoom
        if pos.y < -1 {
            return Chunk::Empty;
        }
        if pos.y >= 1 {
            return Chunk::Empty;
        }

        let perlin = Perlin::new(self.seed);
        let chunk = if pos.y == -1 {
            let mut chunk = chunks::filled_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap());
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

/// A flat water world with solid blocks in the corners of chunks.
#[derive(Debug, Default)]
pub struct Water {
    block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Water {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // zoom doesn't change anything?
        if pos.y == -1 {
            let mut chunk = chunks::filled_chunk(*self.block_mappings.get(WATER_BLOCK_ID).unwrap());
            for x in [0, CHUNK_SIZE - 1] {
                for z in [0, CHUNK_SIZE - 1] {
                    chunk.insert(
                        &BlockPosition {
                            x,
                            y: CHUNK_SIZE - 1,
                            z,
                        },
                        *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                    );
                }
            }
            chunk.into()
        } else {
            Chunk::Empty
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Alternating {
    block_mappings: HashMap<BlockId, ChunkBlockId>,
}

impl WorldGen for Alternating {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }
    fn get(&self, pos: &ChunkPosition, _zoom: f64) -> Chunk {
        // TODO: implement zoom?
        if pos.y % 2 == 0 {
            if pos.x % 2 == 0 || pos.z % 2 == 0 {
                Chunk::Empty
            } else {
                chunks::filled_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
            }
        } else if pos.x % 2 == 0 || pos.z % 2 == 0 {
            chunks::filled_chunk(*self.block_mappings.get(DIRT_BLOCK_ID).unwrap()).into()
        } else {
            Chunk::Empty
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use crate::extras::{block_ids::default_block_ids, chunks::filled_chunk};

    use super::*;

    #[bench]
    fn bench_filled_chunk(b: &mut Bencher) {
        b.iter(|| filled_chunk(0));
    }

    #[bench]
    fn bench_flat(b: &mut Bencher) {
        let mut wgen = Flat::default();
        wgen.initialize(default_block_ids());
        let underground = ChunkPosition { x: 0, y: -1, z: 0 };
        b.iter(|| wgen.get(&underground, 1.));
    }

    #[bench]
    fn bench_perlin_noise(b: &mut Bencher) {
        let mut wgen = PerlinNoise::default();
        wgen.initialize(default_block_ids());
        b.iter(|| wgen.get(&ChunkPosition::default(), 1.));
    }
}
