use std::sync::Arc;

use infinigen_common::blocks::Palette;
use infinigen_common::world::WorldGen;
use serde::{Deserialize, Serialize};
use strum::EnumString;

pub mod flat;
pub mod mountain_islands;
pub mod single_block;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString)]
pub(crate) enum WorldGenTypes {
    Flat,
    #[default]
    MountainIslands,
    SingleBlock,
}

impl WorldGenTypes {
    pub(crate) fn as_world_gen(
        &self,
        seed: u32,
        palette: Palette,
    ) -> Arc<dyn WorldGen + Send + Sync> {
        match self {
            WorldGenTypes::Flat => Arc::new(flat::Flat::from(palette)),
            WorldGenTypes::MountainIslands => {
                Arc::new(mountain_islands::MountainIslands::new(seed, palette))
            }
            WorldGenTypes::SingleBlock => Arc::new(single_block::SingleBlock::from(palette)),
        }
    }
}
