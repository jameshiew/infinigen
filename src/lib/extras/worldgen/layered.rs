use crate::common::chunks::{Chunk, UnpackedChunk, CHUNK_SIZE, CHUNK_SIZE_F64, CHUNK_USIZE};
use crate::common::world::{
    BlockId, BlockPosition, ChunkBlockId, ChunkPosition, WorldGen, WorldPosition,
};
use crate::common::zoom::ZoomLevel;
use crate::extras::block_ids::{GRASS_BLOCK_ID, STONE_BLOCK_ID};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use splines::{Interpolation, Key, Spline};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Layered {
    pub block_mappings: HashMap<BlockId, ChunkBlockId>,
    heightmap: Fbm<Perlin>,
    vertical_scale: f64,
    horizontal_smoothness: f64,
    verticality: Perlin,
    vspline: Spline<f64, f64>,
}

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
            block_mappings: Default::default(),
            vertical_scale: CHUNK_SIZE_F64 * 4.,
            horizontal_smoothness: CHUNK_SIZE_F64 * 0.1,
            heightmap: Fbm::<Perlin>::new(seed).set_octaves(6),
            verticality: Perlin::new(seed),
            vspline,
        }
    }

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

    fn get_terrain(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> UnpackedChunk {
        let zoom = zoom_level.as_f64();
        let mut chunk = UnpackedChunk::default();
        let offset: WorldPosition = pos.into();
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

                        // ensure we fill blocks up to the wheight
                        let block = *self.block_mappings.get(STONE_BLOCK_ID).unwrap();
                        if wy <= wheights[x as usize][z as usize] {
                            chunk.insert(&BlockPosition { x, y, z }, block);
                        }
                    }
                }
            }
        }
        chunk
    }

    fn surface(&self, chunk: &mut UnpackedChunk) {
        // grass
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let mut y = CHUNK_SIZE - 1;
                while y >= 0 {
                    let pos = BlockPosition { x, y, z };
                    if chunk.get(&pos).is_some() {
                        chunk.insert(&pos, *self.block_mappings.get(GRASS_BLOCK_ID).unwrap());
                        break;
                    }
                    y -= 1;
                }
            }
        }
    }
}

impl WorldGen for Layered {
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>) {
        self.block_mappings = mappings;
    }

    fn get(&mut self, pos: &ChunkPosition, zoom: ZoomLevel) -> Chunk {
        let mut chunk = self.get_terrain(pos, zoom);
        if chunk.is_empty() {
            return Chunk::Empty;
        }
        self.surface(&mut chunk);
        chunk.into()
    }
}
