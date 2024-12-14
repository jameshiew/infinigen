use infinigen_common::blocks::{BlockType, BlockVisibility};

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

pub fn block_types() -> impl Iterator<Item = BlockType> {
    [
        BlockType {
            id: STONE_BLOCK_ID.to_string(),
            color: [128, 128, 128, 255],
            ..Default::default()
        },
        BlockType {
            id: DIRT_BLOCK_ID.to_string(),
            color: [139, 69, 19, 255],
            ..Default::default()
        },
        BlockType {
            id: GRASS_BLOCK_ID.to_string(),
            color: [34, 139, 34, 255],
            ..Default::default()
        },
        BlockType {
            id: WATER_BLOCK_ID.to_string(),
            visibility: BlockVisibility::Translucent,
            color: [25, 153, 230, 128],
        },
        BlockType {
            id: LAVA_BLOCK_ID.to_string(),
            visibility: BlockVisibility::Translucent,
            color: [207, 16, 32, 128],
        },
        BlockType {
            id: SAND_BLOCK_ID.to_string(),
            color: [194, 178, 128, 255],
            ..Default::default()
        },
        BlockType {
            id: SNOW_BLOCK_ID.to_string(),
            color: [255, 250, 250, 255],
            ..Default::default()
        },
        BlockType {
            id: WOOD_BLOCK_ID.to_string(),
            color: [139, 69, 19, 255],
            ..Default::default()
        },
        BlockType {
            id: LEAVES_BLOCK_ID.to_string(),
            color: [84, 161, 66, 255],
            ..Default::default()
        },
        BlockType {
            id: GRAVEL_BLOCK_ID.to_string(),
            color: [128, 128, 128, 255],
            ..Default::default()
        },
    ]
    .into_iter()
}
