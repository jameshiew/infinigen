use serde::{Deserialize, Serialize};
use strum::EnumCount;

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

pub type BlockId = String;
pub type BlockColor = [u8; 4];

pub struct BlockType {
    pub id: BlockId,
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
