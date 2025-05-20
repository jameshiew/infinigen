use infinigen_common::blocks::Palette;
use infinigen_common::chunks::Array3Chunk;
use infinigen_common::world::{ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use crate::blocks::DIRT_BLOCK_ID;

/// Generates a completely flat world of dirt at y=0.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Flat {
    dirt: MappedBlockID,
}

#[allow(clippy::fallible_impl_from)]
impl From<Palette> for Flat {
    fn from(palette: Palette) -> Self {
        Self {
            dirt: *palette.inner.get(DIRT_BLOCK_ID).unwrap(),
        }
    }
}

impl WorldGen for Flat {
    fn get(&self, pos: &ChunkPosition, _zoom_level: ZoomLevel) -> Option<Array3Chunk> {
        // zoom level does not change anything
        if pos.y == -1 {
            Some(infinigen_common::chunks::top_chunk(self.dirt))
        } else {
            None
        }
    }
}
