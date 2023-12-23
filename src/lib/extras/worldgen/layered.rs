use crate::common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
use crate::common::world::{
    BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldBlockPosition, WorldGen,
    WorldPosition,
};
use crate::common::zoom::ZoomLevel;
use crate::extras::block_ids::{
    GRASS_BLOCK_ID, GRAVEL_BLOCK_ID, SAND_BLOCK_ID, STONE_BLOCK_ID, WATER_BLOCK_ID,
};
use lru::LruCache;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use splines::{Interpolation, Key, Spline};
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Layered attempts to generate a world using passes (see <https://www.youtube.com/watch?v=YyVAaJqYAfE>)
#[derive(Debug)]
pub struct Layered {
    terrain_cache: HashMap<ZoomLevel, LruCache<ChunkPosition, Chunk>>,
    config: Config,
}

const SEA_LEVEL: f64 = 0.;

impl Default for Layered {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Layered {
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
        Layered {
            terrain_cache: Default::default(),
            config: Config {
                block_mappings: Default::default(),
                vertical_scale: CHUNK_SIZE_F64 * 4.,
                horizontal_smoothness: CHUNK_SIZE_F64 * 0.1,
                heightmap: Fbm::<Perlin>::new(seed).set_octaves(6),
                verticality: Perlin::new(seed),
                vspline,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
    heightmap: Fbm<Perlin>,
    vertical_scale: f64,
    horizontal_smoothness: f64,
    verticality: Perlin,
    vspline: Spline<f64, f64>,
}

impl Config {
    fn get_wheights(&self, pos: &ChunkPosition, zoom: f64) -> [[f64; CHUNK_USIZE]; CHUNK_USIZE] {
        let mut wheights = [[0.; CHUNK_USIZE]; CHUNK_USIZE];
        let offset: WorldPosition = pos.into();
        let zoomed_offset = [
            offset.x as f64 / zoom,
            offset.y as f64 / zoom,
            offset.z as f64 / zoom,
        ];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let wx = x as f64 / zoom + zoomed_offset[0];
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
            }
        }
        wheights
    }

    fn generate_terrain(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk {
        let zoom = zoom_level.as_f64();
        let mut unpacked = UnpackedChunk::default();
        let mut is_empty = true;
        let offset: WorldBlockPosition = pos.into();
        let zoomed_offset = [
            offset.x as f64 / zoom,
            offset.y as f64 / zoom,
            offset.z as f64 / zoom,
        ];

        let wheights = self.get_wheights(pos, zoom);

        {
            let _span = tracing::debug_span!("worldgen{stage = terrain}").entered();
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let wy = y as f64 / zoom + zoomed_offset[1];
                        let wheight = wheights[x as usize][z as usize];

                        // ensure we fill blocks up to the world height
                        let block = *self.block_mappings.get(STONE_BLOCK_ID).unwrap();
                        if wy < wheight {
                            is_empty = false;
                            unpacked.insert(&BlockPosition { x, y, z }, block);
                        }
                    }
                }
            }
        }
        if is_empty {
            Chunk::Empty
        } else {
            Chunk::Unpacked(Box::new(unpacked))
        }
    }

    pub fn layer(&self, pos: &ChunkPosition, zoom_level: ZoomLevel, unpacked: &mut UnpackedChunk) {
        let zoom = zoom_level.as_f64();
        let offset: WorldBlockPosition = pos.into();
        let zoomed_offset = [
            offset.x as f64 / zoom,
            offset.y as f64 / zoom,
            offset.z as f64 / zoom,
        ];

        let wheights = self.get_wheights(pos, zoom);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let wy = y as f64 / zoom + zoomed_offset[1];
                    if unpacked.get(&BlockPosition { x, y, z }).is_none() {
                        if wy <= SEA_LEVEL {
                            unpacked.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(WATER_BLOCK_ID).unwrap(),
                            );
                        }
                        continue;
                    }

                    let wheight = wheights[x as usize][z as usize];
                    if wy as i64 == wheight as i64 {
                        // top layer block
                        if wy < SEA_LEVEL {
                            unpacked.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(GRAVEL_BLOCK_ID).unwrap(),
                            );
                        } else if wy as i64 == SEA_LEVEL as i64 {
                            unpacked.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(SAND_BLOCK_ID).unwrap(),
                            );
                        } else if wy >= SEA_LEVEL {
                            unpacked.insert(
                                &BlockPosition { x, y, z },
                                *self.block_mappings.get(GRASS_BLOCK_ID).unwrap(),
                            );
                        }
                    }
                }
            }
        }
    }
}

impl WorldGen for Layered {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.config.block_mappings = mappings;
    }

    fn get(&mut self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk {
        if self.terrain_cache.get(&zoom_level).is_none() {
            self.terrain_cache
                .insert(zoom_level, LruCache::new(NonZeroUsize::new(1024).unwrap()));
        }
        let zoom_cache = self.terrain_cache.get_mut(&zoom_level).unwrap();
        let mut terrain = zoom_cache
            .get_or_insert(*pos, || self.config.generate_terrain(pos, zoom_level))
            .clone();
        match terrain {
            Chunk::Empty => {
                let mut unpacked = Box::<UnpackedChunk>::default();
                self.config.layer(pos, zoom_level, &mut unpacked);
                Chunk::Unpacked(unpacked)
            }
            Chunk::Unpacked(ref mut unpacked) => {
                self.config.layer(pos, zoom_level, unpacked);
                Chunk::Unpacked(unpacked.clone())
            }
        }
    }
}
