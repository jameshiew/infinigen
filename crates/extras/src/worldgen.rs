use infinigen_common::world::WorldGen;
use serde::{Deserialize, Serialize};

pub mod flat;
pub mod mountain_islands;
pub mod single_block;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldGenTypes {
    Flat,
    #[default]
    MountainIslands,
    SingleBlock,
}

impl From<WorldGenTypes> for Box<dyn WorldGen + Send + Sync> {
    fn from(value: WorldGenTypes) -> Self {
        match value {
            WorldGenTypes::Flat => Box::<flat::Flat>::default(),
            WorldGenTypes::MountainIslands => Box::<mountain_islands::MountainIslands>::default(),
            WorldGenTypes::SingleBlock => Box::<single_block::SingleBlock>::default(),
        }
    }
}
