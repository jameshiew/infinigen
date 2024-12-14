use ahash::AHashMap;
use infinigen_common::blocks::BlockId;
use infinigen_common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
use infinigen_common::world::{
    BlockPosition, ChunkBlockId, ChunkPosition, WorldGen, WorldPosition,
};
use infinigen_common::zoom::ZoomLevel;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::blocks::{GRASS_BLOCK_ID, GRAVEL_BLOCK_ID, SAND_BLOCK_ID, WATER_BLOCK_ID};

pub struct Experiment1 {
    pub block_mappings: AHashMap<BlockId, ChunkBlockId>,
    heightmap: Fbm<Perlin>,
    // bigger = higher
    vertical_scale: f64,
    horizontal_scale: f64,
    sea_level: f64,
}

impl Default for Experiment1 {
    fn default() -> Self {
        Self {
            block_mappings: AHashMap::default(),
            heightmap: Fbm::new(0)
                .set_octaves(6)
                .set_frequency(1.)
                .set_lacunarity(2.)
                .set_persistence(0.8),
            vertical_scale: CHUNK_SIZE_F64 * 2.,
            horizontal_scale: CHUNK_SIZE_F64 * 20.,
            sea_level: 0.,
        }
    }
}

impl WorldGen for Experiment1 {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk {
        let zoom = zoom_level.as_f64();
        let mut chunk = UnpackedChunk::default();
        let mut is_empty = true;
        let offset: WorldPosition = pos.into();

        let mut wheights = [[0.; CHUNK_USIZE]; CHUNK_USIZE];

        // TODO: should these values be adjusted?
        let vertical_scale = self.vertical_scale;

        // carve ground
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let wx = (x as f64 + offset.x as f64) / zoom;
                    let wy = (y as f64 + offset.y as f64) / zoom;
                    let wz = (z as f64 + offset.z as f64) / zoom;

                    // sampled points can't be integral, otherwise they'll be 0
                    let nx = wx / self.horizontal_scale;
                    let nz = wz / self.horizontal_scale;
                    // let nx = wx / self.horizontal_scale;
                    // let nz = wz / self.horizontal_scale;

                    // get approximate height of the world at this wx, wz
                    let wheight = vertical_scale * self.heightmap.get([nx, nz]);

                    wheights[x as usize][z as usize] = wheight;

                    // wheight is sunken, so we're in a body of water
                    if wheight <= wy && wy <= self.sea_level {
                        chunk.insert(
                            &BlockPosition { x, y, z },
                            *self.block_mappings.get(WATER_BLOCK_ID).unwrap(),
                        );

                        is_empty = false;
                        continue;
                    }

                    // ensure we fill blocks up to the wheight
                    if wy <= wheight {
                        if wy < self.sea_level {
                            chunk.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(GRAVEL_BLOCK_ID).unwrap(),
                            );
                        } else if wy.floor() <= (self.sea_level + 1.).floor() {
                            // sand always borders water
                            chunk.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(SAND_BLOCK_ID).unwrap(),
                            );
                        } else {
                            chunk.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                            );
                        }

                        is_empty = false;
                        continue;
                    }
                }
            }
        }
        if is_empty {
            return Chunk::Empty;
        }

        chunk.into()
    }
}
