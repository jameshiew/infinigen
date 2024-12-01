use bevy::asset::{Asset, Handle, LoadedFolder};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Resource, TypePath};
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
        tracing::info!(?block_definition, ?mapped_id, "Mapping block");
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
