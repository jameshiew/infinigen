use std::sync::Arc;

use infinigen_common::blocks::Palette;
use infinigen_common::world::WorldGen;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub mod flat;
pub mod mountain_islands;
pub mod single_block;

#[derive(
    Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display,
)]
pub enum WorldGenTypes {
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
            Self::Flat => Arc::new(flat::Flat::from(palette)),
            Self::MountainIslands => {
                Arc::new(mountain_islands::MountainIslands::new(seed, palette))
            }
            Self::SingleBlock => Arc::new(single_block::SingleBlock::from(palette)),
        }
    }
}
