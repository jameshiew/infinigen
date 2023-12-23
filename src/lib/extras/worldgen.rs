use serde::{Deserialize, Serialize};

use crate::common::world::WorldGen;

mod alternating;
mod bordered_towers;
mod bowl;
pub mod experiment1;
pub mod flat;
mod layered;
pub mod mountain_archipelago;
mod perlin_noise;
mod random;
mod single_block;
mod water;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldGenTypes {
    #[default]
    Flat,
    BorderedTowers,
    Bowl,
    Random,
    PerlinNoise,
    Water,
    MountainIslands,
    Alternating,
    SingleBlock,
    Experiment1,
    Layered,
}

impl From<WorldGenTypes> for Box<dyn WorldGen + Send + Sync> {
    fn from(value: WorldGenTypes) -> Self {
        match value {
            WorldGenTypes::Flat => Box::<flat::Flat>::default(),
            WorldGenTypes::BorderedTowers => Box::<bordered_towers::BorderedTowers>::default(),
            WorldGenTypes::Random => Box::<random::Random>::default(),
            WorldGenTypes::PerlinNoise => Box::<perlin_noise::PerlinNoise>::default(),
            WorldGenTypes::Water => Box::<water::Water>::default(),
            WorldGenTypes::MountainIslands => {
                Box::<mountain_archipelago::MountainIslands>::default()
            }
            WorldGenTypes::Alternating => Box::<alternating::Alternating>::default(),
            WorldGenTypes::SingleBlock => Box::<single_block::SingleBlock>::default(),
            WorldGenTypes::Experiment1 => Box::<experiment1::Experiment1>::default(),
            WorldGenTypes::Bowl => Box::<bowl::Bowl>::default(),
            WorldGenTypes::Layered => Box::<layered::Layered>::default(),
        }
    }
}
