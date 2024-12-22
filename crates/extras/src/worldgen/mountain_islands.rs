use infinigen_common::blocks::Palette;
use infinigen_common::chunks::{Array3Chunk, Chunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
use infinigen_common::world::{
    BlockPosition, ChunkPosition, MappedBlockID, WorldGen, WorldPosition,
};
use infinigen_common::zoom::ZoomLevel;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use splines::{Interpolation, Key, Spline};

use crate::blocks::{
    DIRT_BLOCK_ID, GRASS_BLOCK_ID, GRAVEL_BLOCK_ID, SAND_BLOCK_ID, SNOW_BLOCK_ID, STONE_BLOCK_ID,
    WATER_BLOCK_ID,
};

#[derive(Debug, Clone)]
pub struct MountainIslands {
    /// The world height at any given (x, z)
    heightmap: Fbm<Perlin>,
    verticality: Perlin,
    terrain_variance: Fbm<Perlin>,
    vspline: Spline<f64, f64>,
    /// max mountain size without zoom is roughly double this value
    vertical_scale: f64,
    horizontal_smoothness: f64,

    water: MappedBlockID,
    snow: MappedBlockID,
    gravel: MappedBlockID,
    sand: MappedBlockID,
    dirt: MappedBlockID,
    grass: MappedBlockID,
    stone: MappedBlockID,
}

impl From<Palette> for MountainIslands {
    fn from(palette: Palette) -> Self {
        let mut wgen = Self::default();
        tracing::debug!(?wgen.heightmap.octaves, wgen.heightmap.frequency, wgen.heightmap.lacunarity, wgen.heightmap.persistence, "MountainIslands initialized");

        wgen.water = *palette.inner.get(WATER_BLOCK_ID).unwrap();
        wgen.snow = *palette.inner.get(SNOW_BLOCK_ID).unwrap();
        wgen.gravel = *palette.inner.get(GRAVEL_BLOCK_ID).unwrap();
        wgen.sand = *palette.inner.get(SAND_BLOCK_ID).unwrap();
        wgen.dirt = *palette.inner.get(DIRT_BLOCK_ID).unwrap();
        wgen.grass = *palette.inner.get(GRASS_BLOCK_ID).unwrap();
        wgen.stone = *palette.inner.get(STONE_BLOCK_ID).unwrap();
        wgen
    }
}

impl Default for MountainIslands {
    fn default() -> Self {
        let seed = 0;
        let vspline = Spline::from_vec(vec![
            Key::new(-1., 0.6, Interpolation::Cosine),
            Key::new(-0.9, 0.7, Interpolation::Cosine),
            Key::new(0., 0.8, Interpolation::Cosine),
            Key::new(0.5, 0.85, Interpolation::Cosine),
            Key::new(0.8, 0.9, Interpolation::Cosine),
            Key::new(0.9, 1., Interpolation::Cosine),
            Key::new(1.1, 1.5, Interpolation::default()), // this last one must be strictly greater than 1 because sometime we may sample with exactly the value 1.
        ]);
        Self {
            heightmap: default_heightmap(seed),
            verticality: Perlin::new(seed),
            terrain_variance: default_terrain_variance(seed),
            vspline,
            vertical_scale: CHUNK_SIZE_F64 * 4.,
            horizontal_smoothness: CHUNK_SIZE_F64 * 0.1,

            water: 1.try_into().unwrap(),
            snow: 2.try_into().unwrap(),
            gravel: 3.try_into().unwrap(),
            sand: 4.try_into().unwrap(),
            dirt: 5.try_into().unwrap(),
            grass: 6.try_into().unwrap(),
            stone: 7.try_into().unwrap(),
        }
    }
}

fn default_heightmap(seed: u32) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed).set_octaves(6)
}

pub fn default_terrain_variance(seed: u32) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed).set_octaves(8).set_persistence(0.7)
}

const SEA_LEVEL: f64 = 0.;

// we still bound the worldgen on the Y axis to improve performance
// for an infinitely deep world, we would not have a MIN_Y_HEIGHT maybe
const MIN_Y_HEIGHT: i32 = -6;

/// Based on <https://www.youtube.com/watch?v=CSa5O6knuwI>
impl WorldGen for MountainIslands {
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

        let mut chunk = Array3Chunk::default();
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
