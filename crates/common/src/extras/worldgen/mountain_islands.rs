use ahash::AHashMap;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use splines::{Interpolation, Key, Spline};

use crate::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
use crate::extras::block_ids::{
    DIRT_BLOCK_ID, GRASS_BLOCK_ID, GRAVEL_BLOCK_ID, SAND_BLOCK_ID, SNOW_BLOCK_ID, STONE_BLOCK_ID,
    WATER_BLOCK_ID,
};
use crate::world::{BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen, WorldPosition};
use crate::zoom::ZoomLevel;

pub struct MountainIslands {
    pub block_mappings: AHashMap<BlockId, ChunkBlockId>,
    /// The world height at any given (x, z)
    heightmap: Fbm<Perlin>,
    verticality: Perlin,
    terrain_variance: Fbm<Perlin>,
    vspline: Spline<f64, f64>,
    /// max mountain size without zoom is roughly double this value
    vertical_scale: f64,
    horizontal_smoothness: f64,
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
            block_mappings: Default::default(),
            heightmap: default_heightmap(seed),
            verticality: Perlin::new(seed),
            terrain_variance: default_terrain_variance(seed),
            vspline,
            vertical_scale: CHUNK_SIZE_F64 * 4.,
            horizontal_smoothness: CHUNK_SIZE_F64 * 0.1,
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
        self.block_mappings = mappings;
    }

    fn get(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk {
        if pos.y < MIN_Y_HEIGHT {
            return Chunk::Empty;
        }
        let zoom = zoom_level.as_f64();
        // let snow_level: f64 = (SEA_LEVEL + self.vertical_scale) * zoom;

        let mut chunk = UnpackedChunk::default();
        let mut is_empty = true;
        let offset: WorldPosition = pos.into();
        let zoomed_offset = [
            offset.x as f64 / zoom,
            offset.y as f64 / zoom,
            offset.z as f64 / zoom,
        ];

        // TODO: only record wheights that are in this chunk, as we only decorate if the wheight is in our chunk
        let mut wheights = [[0.; CHUNK_USIZE]; CHUNK_USIZE];

        {
            let _span = tracing::debug_span!("worldgen{stage = terrain}").entered();
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let wx = x as f64 / zoom + zoomed_offset[0];
                        let wy = y as f64 / zoom + zoomed_offset[1];
                        let wz = z as f64 / zoom + zoomed_offset[2];

                        let nx = wx / (self.horizontal_smoothness * self.vertical_scale);
                        let nz = wz / (self.horizontal_smoothness * self.vertical_scale);

                        // get approximate height of the world at this wx, wz
                        let mut wheight =
                            tracing::trace_span!("worldgen{stage = terrain, noise = heightmap}")
                                .in_scope(|| self.heightmap.get([nx, nz]));

                        let verticality =
                            tracing::trace_span!("worldgen{stage = terrain, noise = verticality}")
                                .in_scope(|| self.verticality.get([nx, nz]));
                        wheight *= self.vertical_scale * self.vspline.sample(verticality).unwrap();

                        wheights[x as usize][z as usize] = wheight;

                        // wheight is sunken, so we're in a body of water
                        if wheight <= wy && wy <= SEA_LEVEL {
                            is_empty = false;
                            chunk.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(WATER_BLOCK_ID).unwrap(),
                            );
                            continue;
                        }

                        // ensure we fill blocks up to the wheight
                        if wy <= wheight {
                            is_empty = false;
                            if wy < SEA_LEVEL {
                                // always gravel under sea
                                chunk.insert(
                                    &BlockPosition { x, y, z },
                                    *self.block_mappings.get(GRAVEL_BLOCK_ID).unwrap(),
                                );
                                continue;
                            } else if wy.floor() <= (SEA_LEVEL + (1. / zoom)).floor() {
                                // sand always borders water
                                chunk.insert(
                                    &BlockPosition { x, y, z },
                                    *self.block_mappings.get(SAND_BLOCK_ID).unwrap(),
                                );
                                continue;
                            }

                            let next_band_chance = self.terrain_variance.get([nx, nz]) / 2.;

                            let block_ranges = [
                                (SEA_LEVEL + (-3. * zoom + 1.), SAND_BLOCK_ID),
                                (SEA_LEVEL + (9. * zoom + 1.), DIRT_BLOCK_ID),
                                (SEA_LEVEL + (285. * zoom + 1.), GRASS_BLOCK_ID),
                                (SEA_LEVEL + (300. * zoom + 1.), STONE_BLOCK_ID),
                                (f64::INFINITY, SNOW_BLOCK_ID),
                            ];

                            // Assign block type based on the height and noise.
                            let mut block_id = block_ranges[0].1;
                            for &(threshold, id) in &block_ranges {
                                if wy + next_band_chance * self.vertical_scale < threshold {
                                    block_id = id;
                                    break;
                                }
                            }

                            let block = *self.block_mappings.get(block_id).unwrap();
                            chunk.insert(&BlockPosition { x, y, z }, block);
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
