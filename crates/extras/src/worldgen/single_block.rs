use infinigen_common::blocks::Palette;
use infinigen_common::chunks::{Array3Chunk, CHUNK_SIZE};
use infinigen_common::world::{BlockPosition, ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::blocks::GRASS_BLOCK_ID;

/// Generates a single block in the middle of every chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleBlock {
    grass: MappedBlockID,
}

#[allow(clippy::fallible_impl_from)]
impl From<Palette> for SingleBlock {
    fn from(palette: Palette) -> Self {
        Self {
            grass: *palette.inner.get(GRASS_BLOCK_ID).unwrap(),
        }
    }
}

impl WorldGen for SingleBlock {
    fn get(&self, _pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Option<Array3Chunk> {
        // TODO: implement zoom?
        let mut chunk = Array3Chunk::default();
        chunk.insert(
            &BlockPosition {
                x: CHUNK_SIZE / 2,
                y: 0,
                z: CHUNK_SIZE / 2,
            },
            self.grass,
        );
        Some(chunk)
    }
}
