//! IDs of the default blocks provided in this repo.

use rustc_hash::FxHashMap;

use crate::common::world::{BlockId, ChunkBlockId};

pub const DIRT_BLOCK_ID: &str = "infinigen:dirt";
pub const GRASS_BLOCK_ID: &str = "infinigen:grass";
pub const GRAVEL_BLOCK_ID: &str = "infinigen:gravel";
pub const LAVA_BLOCK_ID: &str = "infinigen:lava";
pub const LEAVES_BLOCK_ID: &str = "infinigen:leaves";
pub const SAND_BLOCK_ID: &str = "infinigen:sand";
pub const SNOW_BLOCK_ID: &str = "infinigen:snow";
pub const STONE_BLOCK_ID: &str = "infinigen:stone";
pub const WATER_BLOCK_ID: &str = "infinigen:water";
pub const WOOD_BLOCK_ID: &str = "infinigen:wood";

pub fn default_block_ids() -> FxHashMap<BlockId, ChunkBlockId> {
    FxHashMap::from_iter(vec![
        (DIRT_BLOCK_ID.to_owned(), 1),
        (GRASS_BLOCK_ID.to_owned(), 2),
        (GRAVEL_BLOCK_ID.to_owned(), 3),
        (LAVA_BLOCK_ID.to_owned(), 4),
        (LEAVES_BLOCK_ID.to_owned(), 5),
        (SAND_BLOCK_ID.to_owned(), 6),
        (SNOW_BLOCK_ID.to_owned(), 7),
        (STONE_BLOCK_ID.to_owned(), 8),
        (WATER_BLOCK_ID.to_owned(), 9),
        (WOOD_BLOCK_ID.to_owned(), 10),
    ])
}
