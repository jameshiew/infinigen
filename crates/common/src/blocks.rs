use std::collections::BTreeMap;

use ahash::AHashMap;
use linearize::Linearize;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter};

use crate::world::MappedBlockID;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    EnumCount,
)]
pub enum BlockVisibility {
    #[default]
    Opaque = 0,
    Translucent,
}

pub type BlockID = String;
pub type BlockColor = [u8; 4];

type TextureFilename = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct BlockType {
    pub id: BlockID,
    #[serde(default)]
    pub visibility: BlockVisibility,
    #[serde(default = "default_block_color")]
    pub color: BlockColor,
    pub textures: Option<BTreeMap<Face, TextureFilename>>,
}

impl Default for BlockType {
    fn default() -> Self {
        Self {
            color: default_block_color(),
            id: "default".to_string(),
            visibility: BlockVisibility::Opaque,
            textures: None,
        }
    }
}

const fn default_block_color() -> BlockColor {
    [255, 255, 255, 255]
}

#[derive(Debug, Default)]
pub struct Palette {
    pub inner: AHashMap<BlockID, MappedBlockID>,
}

impl From<AHashMap<BlockID, MappedBlockID>> for Palette {
    fn from(value: AHashMap<BlockID, MappedBlockID>) -> Self {
        Self { inner: value }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    EnumIter,
    Linearize,
)]
#[linearize(const)]
pub enum Face {
    Top = 0,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}
