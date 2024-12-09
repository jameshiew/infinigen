use ahash::AHashMap;
use infinigen_common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
use infinigen_common::world::{
    BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen, WorldPosition,
};
use infinigen_common::zoom::ZoomLevel;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use splines::{Interpolation, Key, Spline};

use crate::block_ids::{
    DIRT_BLOCK_ID, GRASS_BLOCK_ID, GRAVEL_BLOCK_ID, SAND_BLOCK_ID, SNOW_BLOCK_ID, STONE_BLOCK_ID,
    WATER_BLOCK_ID,
};

pub struct MountainIslands {
    /// The world height at any given (x, z)
    heightmap: Fbm<Perlin>,
    verticality: Perlin,
    terrain_variance: Fbm<Perlin>,
    vspline: Spline<f64, f64>,
    /// max mountain size without zoom is roughly double this value
    vertical_scale: f64,
    horizontal_smoothness: f64,

    water: ChunkBlockId,
    snow: ChunkBlockId,
    gravel: ChunkBlockId,
    sand: ChunkBlockId,
    dirt: ChunkBlockId,
    grass: ChunkBlockId,
    stone: ChunkBlockId,
}

impl MountainIslands {
    pub fn new(seed: u32) -> Self {
        let vspline = Spline::from_vec(vec![
            Key::new(-1., 0.6, Interpolation::Cosine),
            Key::new(-0.9, 0.7, Interpolation::Cosine),
            Key::new(0., 0.8, Interpolation::Cosine),
            Key::new(0.5, 0.85, Interpolation::Cosine),
            Key::new(0.8, 0.9, Interpolation::Cosine),
            Key::new(0.9, 1., Interpolation::Cosine),
            Key::new(1.1, 1.5, Interpolation::default()), // this last one must be strictly greater than 1 because sometime we may sample with exactly the value 1.
        ]);

        let wgen = Self {
            heightmap: default_heightmap(seed),
            verticality: Perlin::new(seed),
            terrain_variance: default_terrain_variance(seed),
            vspline,
            vertical_scale: CHUNK_SIZE_F64 * 4.,
            horizontal_smoothness: CHUNK_SIZE_F64 * 0.1,

            water: 1,
            snow: 2,
            gravel: 3,
            sand: 4,
            dirt: 5,
            grass: 6,
            stone: 7,
        };
        tracing::debug!(?wgen.heightmap.octaves, wgen.heightmap.frequency, wgen.heightmap.lacunarity, wgen.heightmap.persistence, "MountainIslands initialized");
        wgen
    }
}

fn default_heightmap(seed: u32) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed).set_octaves(6)
}

pub fn default_terrain_variance(seed: u32) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed).set_octaves(8).set_persistence(0.7)
}

impl Default for MountainIslands {
    fn default() -> Self {
        Self::new(0)
    }
}

const SEA_LEVEL: f64 = 0.;

// we still bound the worldgen on the Y axis to improve performance
// for an infinitely deep world, we would not have a MIN_Y_HEIGHT maybe
const MIN_Y_HEIGHT: i32 = -6;

/// Based on <https://www.youtube.com/watch?v=CSa5O6knuwI>
impl WorldGen for MountainIslands {
    fn initialize(&mut self, mappings: AHashMap<BlockId, ChunkBlockId>) {
        self.water = *mappings.get(WATER_BLOCK_ID).unwrap();
        self.snow = *mappings.get(SNOW_BLOCK_ID).unwrap();
        self.gravel = *mappings.get(GRAVEL_BLOCK_ID).unwrap();
        self.sand = *mappings.get(SAND_BLOCK_ID).unwrap();
        self.dirt = *mappings.get(DIRT_BLOCK_ID).unwrap();
        self.grass = *mappings.get(GRASS_BLOCK_ID).unwrap();
        self.stone = *mappings.get(STONE_BLOCK_ID).unwrap();
    }

    fn get(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk {
        if pos.y < MIN_Y_HEIGHT {
            return Chunk::Empty;
        }
        let zoom = zoom_level.as_f64();
        let sand_level = (SEA_LEVEL + (1. / zoom)).floor();
        // let snow_level: f64 = (SEA_LEVEL + self.vertical_scale) * zoom;

        let block_ranges = [
            (SEA_LEVEL + (-3. * zoom + 1.), self.sand),
            (SEA_LEVEL + (9. * zoom + 1.), self.dirt),
            (SEA_LEVEL + (285. * zoom + 1.), self.grass),
            (SEA_LEVEL + (300. * zoom + 1.), self.stone),
            (f64::INFINITY, self.snow),
        ];

        let mut chunk = UnpackedChunk::default();
        let mut is_empty = true;
        let offset: WorldPosition = pos.into();
        let zoomed_offset = [
            offset.x as f64 / zoom,
            offset.y as f64 / zoom,
            offset.z as f64 / zoom,
        ];

        let mut terrain_variances = [[0.; CHUNK_USIZE]; CHUNK_USIZE];

        {
            let _span = tracing::debug_span!("worldgen{stage = terrain}").entered();
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let wx = x as f64 / zoom + zoomed_offset[0];
                    let wz = z as f64 / zoom + zoomed_offset[2];

                    let nx = wx / (self.horizontal_smoothness * self.vertical_scale);
                    let nz = wz / (self.horizontal_smoothness * self.vertical_scale);

                    let mut wheight = self.heightmap.get([nx, nz]);
                    let verticality = self.verticality.get([nx, nz]);
                    wheight *= self.vertical_scale * self.vspline.sample(verticality).unwrap();
                    for y in 0..CHUNK_SIZE {
                        let wy = y as f64 / zoom + zoomed_offset[1];

                        // wheight is sunken, so we're in a body of water
                        if wheight <= wy && wy <= SEA_LEVEL {
                            is_empty = false;
                            chunk.insert(&BlockPosition { x, y, z }, self.water);
                            continue;
                        }

                        // ensure we fill blocks up to the wheight
                        if wy <= wheight {
                            is_empty = false;
                            if wy < SEA_LEVEL {
                                // always gravel under sea
                                chunk.insert(&BlockPosition { x, y, z }, self.gravel);
                                continue;
                            } else if wy.floor() <= sand_level {
                                // sand always borders water
                                chunk.insert(&BlockPosition { x, y, z }, self.sand);
                                continue;
                            }

                            let next_band_chance = {
                                let val = &mut terrain_variances[x as usize][z as usize];
                                // exactly float zero means it (probably?) wasn't calculated before
                                if *val == 0. {
                                    *val = self.terrain_variance.get([nx, nz]) / 2.0;
                                }
                                *val
                            };

                            // Assign block type based on the height and noise.
                            let mut block_id = block_ranges[0].1;
                            for &(threshold, id) in &block_ranges {
                                if wy + next_band_chance * self.vertical_scale < threshold {
                                    block_id = id;
                                    break;
                                }
                            }

                            chunk.insert(&BlockPosition { x, y, z }, block_id);
                        }
                    }
                }
            }

            if is_empty {
                return Chunk::Empty;
            }
        }

        chunk.into()
    }
}
