use bevy::asset::{Asset, Handle, LoadedFolder};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Resource, TypePath};
use infinigen_common::extras::block_ids::{
    DIRT_BLOCK_ID, GRASS_BLOCK_ID, GRAVEL_BLOCK_ID, LAVA_BLOCK_ID, LEAVES_BLOCK_ID, SAND_BLOCK_ID,
    SNOW_BLOCK_ID, STONE_BLOCK_ID, WATER_BLOCK_ID, WOOD_BLOCK_ID,
};
use infinigen_common::mesh::faces::BlockVisibilityChecker;
use infinigen_common::mesh::textures::{Face, TextureMap};
use infinigen_common::world::{BlockId, BlockVisibility, ChunkBlockId};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum::EnumCount;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, EnumCount)]
pub enum MaterialType {
    DenseOpaque = 0,
    Translucent,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, TypePath, Asset,
)]
pub struct BlockDefinition {
    pub id: BlockId,
    #[serde(default)]
    pub visibility: BlockVisibility,
    #[serde(default = "default_block_color")]
    pub color: [u8; 4],
    pub textures: Option<BTreeMap<Face, String>>,
}

impl Default for BlockDefinition {
    fn default() -> Self {
        BlockDefinition {
            color: default_block_color(),
            id: "infinigen:default".to_string(),
            visibility: BlockVisibility::Opaque,
            textures: None,
        }
    }
}

fn default_block_color() -> [u8; 4] {
    [255, 255, 255, 255]
}

/// Maps mapped block IDs (i.e. bytes) <-> block definitions.
// TODO: overflow checking, safety
#[derive(Debug, Default, Clone)]
pub struct BlockMappings {
    pub by_mapped_id: FxHashMap<ChunkBlockId, BlockDefinition>,
    by_block_id: FxHashMap<BlockId, ChunkBlockId>,
    next_free_mapped_id: ChunkBlockId,
}

impl From<&BlockMappings> for FxHashMap<BlockId, ChunkBlockId> {
    fn from(value: &BlockMappings) -> Self {
        value.by_block_id.clone()
    }
}

impl BlockMappings {
    pub fn get_by_mapped_id(&self, mapped_id: &ChunkBlockId) -> &BlockDefinition {
        self.by_mapped_id.get(mapped_id).unwrap()
    }

    pub fn get_by_block_id(&self, block_id: &BlockId) -> ChunkBlockId {
        *self.by_block_id.get(block_id).unwrap()
    }

    pub fn add(&mut self, block_definition: BlockDefinition) -> ChunkBlockId {
        let mapped_id = self.next_free_mapped_id;
        tracing::debug!(?block_definition, ?mapped_id, "Mapping block");
        self.by_block_id
            .insert(block_definition.id.clone(), self.next_free_mapped_id);
        self.by_mapped_id
            .insert(self.next_free_mapped_id, block_definition);
        self.next_free_mapped_id += 1;
        mapped_id
    }
}

impl BlockVisibilityChecker for &BlockMappings {
    fn get_visibility(&self, mapped_id: &ChunkBlockId) -> BlockVisibility {
        self.get_by_mapped_id(mapped_id).visibility
    }
}

impl BlockVisibilityChecker for BlockMappings {
    fn get_visibility(&self, mapped_id: &ChunkBlockId) -> BlockVisibility {
        (&self).get_visibility(mapped_id)
    }
}

#[derive(Default, Resource)]
pub struct BlockRegistry {
    pub materials: [Handle<StandardMaterial>; MaterialType::COUNT],
    pub block_texture_folder: Handle<LoadedFolder>,
    pub block_definition_folder: Handle<LoadedFolder>,
    pub block_textures: TextureMap,
    pub block_mappings: BlockMappings,
}

impl BlockRegistry {
    /// Returns a weak handle to a material.
    pub fn get_material(&self, material_type: MaterialType) -> Handle<StandardMaterial> {
        self.materials[material_type as usize].clone_weak()
    }

    /// Returns a weak handle to the block texture atlas.
    pub fn get_block_textures(&self) -> &TextureMap {
        &self.block_textures
    }
}

pub fn default_block_definitions() -> Vec<BlockDefinition> {
    vec![
        // Stone
        BlockDefinition {
            id: STONE_BLOCK_ID.to_string(),
            color: [128, 128, 128, 255],
            ..Default::default()
        },
        // Dirt
        BlockDefinition {
            id: DIRT_BLOCK_ID.to_string(),
            color: [139, 69, 19, 255],
            ..Default::default()
        },
        // Grass
        BlockDefinition {
            id: GRASS_BLOCK_ID.to_string(),
            color: [34, 139, 34, 255],
            ..Default::default()
        },
        // Water
        BlockDefinition {
            id: WATER_BLOCK_ID.to_string(),
            visibility: BlockVisibility::Translucent,
            color: [25, 153, 230, 128],
            ..Default::default()
        },
        // Lava
        BlockDefinition {
            id: LAVA_BLOCK_ID.to_string(),
            visibility: BlockVisibility::Translucent,
            color: [207, 16, 32, 128],
            ..Default::default()
        },
        // Sand
        BlockDefinition {
            id: SAND_BLOCK_ID.to_string(),
            color: [194, 178, 128, 255],
            ..Default::default()
        },
        // Snow
        BlockDefinition {
            id: SNOW_BLOCK_ID.to_string(),
            color: [255, 250, 250, 255],
            ..Default::default()
        },
        // Wood
        BlockDefinition {
            id: WOOD_BLOCK_ID.to_string(),
            color: [139, 69, 19, 255],
            ..Default::default()
        },
        // Leaves
        BlockDefinition {
            id: LEAVES_BLOCK_ID.to_string(),
            color: [84, 161, 66, 255],
            ..Default::default()
        },
        // Gravel
        BlockDefinition {
            id: GRAVEL_BLOCK_ID.to_string(),
            color: [128, 128, 128, 255],
            ..Default::default()
        },
    ]
}
