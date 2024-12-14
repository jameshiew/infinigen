use std::collections::BTreeMap;

use ahash::AHashMap;
use bevy::asset::{Asset, Handle};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Resource, TypePath};
use infinigen_common::blocks::{BlockColor, BlockID, BlockType, BlockVisibility};
use infinigen_common::mesh::faces::BlockVisibilityChecker;
use infinigen_common::mesh::textures::{Face, TextureMap};
use infinigen_common::world::MappedBlockID;
use serde::{Deserialize, Serialize};
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
    pub id: BlockID,
    #[serde(default)]
    pub visibility: BlockVisibility,
    #[serde(default = "default_block_color")]
    pub color: BlockColor,
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

impl From<BlockType> for BlockDefinition {
    fn from(value: BlockType) -> Self {
        Self {
            color: value.color,
            id: value.id,
            visibility: value.visibility,
            ..Default::default()
        }
    }
}

fn default_block_color() -> BlockColor {
    [255, 255, 255, 255]
}

/// Maps mapped block IDs (i.e. bytes) <-> block definitions.
// TODO: overflow checking, safety
#[derive(Debug, Default, Clone)]
pub struct BlockMappings {
    pub by_mapped_id: AHashMap<MappedBlockID, BlockDefinition>,
    by_block_id: AHashMap<BlockID, MappedBlockID>,
    next_free_mapped_id: MappedBlockID,
}

impl From<&BlockMappings> for AHashMap<BlockID, MappedBlockID> {
    fn from(value: &BlockMappings) -> Self {
        value.by_block_id.clone()
    }
}

impl BlockMappings {
    pub fn get_by_mapped_id(&self, mapped_id: &MappedBlockID) -> &BlockDefinition {
        self.by_mapped_id.get(mapped_id).unwrap()
    }

    pub fn get_by_block_id(&self, block_id: &BlockID) -> MappedBlockID {
        *self.by_block_id.get(block_id).unwrap()
    }

    pub fn add(&mut self, block_definition: BlockDefinition) -> MappedBlockID {
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
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility {
        self.get_by_mapped_id(mapped_id).visibility
    }
}

impl BlockVisibilityChecker for BlockMappings {
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility {
        (&self).get_visibility(mapped_id)
    }
}

#[derive(Default, Resource)]
pub struct BlockRegistry {
    pub materials: [Handle<StandardMaterial>; MaterialType::COUNT],
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
