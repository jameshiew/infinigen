use serde::{Deserialize, Serialize};

use crate::common::world::WorldGen;

pub mod alternating;
pub mod bordered_towers;
pub mod bowl;
pub mod experiment1;
pub mod flat;
pub mod layered;
pub mod mountain_archipelago;
pub mod perlin_noise;
pub mod random;
pub mod single_block;
pub mod water;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldGenTypes {
    Flat,
    BorderedTowers,
    Bowl,
    Random,
    PerlinNoise,
    Water,
    #[default]
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
