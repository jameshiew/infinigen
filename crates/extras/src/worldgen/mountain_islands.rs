use infinigen_common::blocks::Palette;
use infinigen_common::chunks::{Array3Chunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
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

impl MountainIslands {
    pub fn new(seed: u32, palette: Palette) -> Self {
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

            water: *palette.inner.get(WATER_BLOCK_ID).unwrap(),
            snow: *palette.inner.get(SNOW_BLOCK_ID).unwrap(),
            gravel: *palette.inner.get(GRAVEL_BLOCK_ID).unwrap(),
            sand: *palette.inner.get(SAND_BLOCK_ID).unwrap(),
            dirt: *palette.inner.get(DIRT_BLOCK_ID).unwrap(),
            grass: *palette.inner.get(GRASS_BLOCK_ID).unwrap(),
            stone: *palette.inner.get(STONE_BLOCK_ID).unwrap(),
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

const SEA_LEVEL: f64 = 0.;

// we still bound the worldgen on the Y axis to improve performance
// for an infinitely deep world, we would not have a MIN_Y_HEIGHT maybe
const MIN_Y_HEIGHT: i32 = -6;

/// Based on <https://www.youtube.com/watch?v=CSa5O6knuwI>
impl WorldGen for MountainIslands {
    fn get(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Option<Array3Chunk> {
        if pos.y < MIN_Y_HEIGHT {
            return None;
        }
        let zoom = zoom_level.as_f64();
        let sand_level = (SEA_LEVEL + (1. / zoom)).floor();
        // let snow_level: f64 = (SEA_LEVEL + self.vertical_scale) * zoom;

        let block_ranges = [
            (SEA_LEVEL + (-3.0f64).mul_add(zoom, 1.), self.sand),
            (SEA_LEVEL + 9.0f64.mul_add(zoom, 1.), self.dirt),
            (SEA_LEVEL + 285.0f64.mul_add(zoom, 1.), self.grass),
            (SEA_LEVEL + 300.0f64.mul_add(zoom, 1.), self.stone),
            (f64::INFINITY, self.snow),
        ];

        let mut chunk = Array3Chunk::default();
        let offset: WorldPosition = pos.into();
        let zoomed_offset = [
            offset.x as f64 / zoom,
            offset.y as f64 / zoom,
            offset.z as f64 / zoom,
        ];

        // needed for every column
        let mut world_heights = [[0.; CHUNK_USIZE]; CHUNK_USIZE];
        let mut nxzs = [[(0., 0.); CHUNK_USIZE]; CHUNK_USIZE];

        let mut is_empty = true;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let wx = x as f64 / zoom + zoomed_offset[0];
                let wz = z as f64 / zoom + zoomed_offset[2];

                let nx = wx / (self.horizontal_smoothness * self.vertical_scale);
                let nz = wz / (self.horizontal_smoothness * self.vertical_scale);

                let mut world_height = self.heightmap.get([nx, nz]);
                let verticality = self.verticality.get([nx, nz]);
                world_height *= self.vertical_scale * self.vspline.sample(verticality).unwrap();

                // short circuit if bottom-most layer (y=0) is empty as this world doesn't have things in the sky
                let wy = zoomed_offset[1];
                if wy <= world_height || wy <= SEA_LEVEL {
                    is_empty = false;
                }

                world_heights[x as usize][z as usize] = world_height;
                nxzs[x as usize][z as usize] = (nx, nz);
            }
        }
        if is_empty {
            return None;
        }

        let mut terrain_variances = [[None::<f64>; CHUNK_USIZE]; CHUNK_USIZE];

        {
            let _span = tracing::debug_span!("worldgen{stage = terrain}").entered();
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let world_height = world_heights[x as usize][z as usize];
                    for y in 0..CHUNK_SIZE {
                        let wy = y as f64 / zoom + zoomed_offset[1];

                        // world_height is sunken, so we're in a body of water
                        if world_height <= wy && wy <= SEA_LEVEL {
                            is_empty = false;
                            chunk.insert(&BlockPosition { x, y, z }, self.water);
                            continue;
                        }

                        // ensure we fill blocks up to the world_height
                        if wy <= world_height {
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
                                let slot = &mut terrain_variances[x as usize][z as usize];
                                let value = slot.get_or_insert_with(|| {
                                    let (nx, nz) = nxzs[x as usize][z as usize];
                                    self.terrain_variance.get([nx, nz]) / 2.0
                                });
                                *value
                            };

                            // Assign block type based on the height and noise.
                            let mut block_id = block_ranges[0].1;
                            for &(threshold, id) in &block_ranges {
                                if next_band_chance.mul_add(self.vertical_scale, wy) < threshold {
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
                return None;
            }
        }

        Some(chunk)
    }
}
