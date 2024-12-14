use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::world::MappedBlockID;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub enum BlockVisibility {
    #[default]
    Opaque = 0,
    Translucent,
}

pub type BlockID = String;
pub type BlockColor = [u8; 4];

pub struct BlockType {
    pub id: BlockID,
    pub visibility: BlockVisibility,
    pub color: BlockColor,
}

impl Default for BlockType {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            visibility: BlockVisibility::Opaque,
            color: [255, 255, 255, 255],
        }
    }
}

pub struct Palette {
    pub inner: AHashMap<BlockID, MappedBlockID>,
}

impl From<AHashMap<BlockID, MappedBlockID>> for Palette {
    fn from(value: AHashMap<BlockID, MappedBlockID>) -> Self {
        Self { inner: value }
    }
}
