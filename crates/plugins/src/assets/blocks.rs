use std::collections::BTreeMap;

use ahash::AHashMap;
use bevy::asset::{Asset, Handle};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Resource, TypePath};
use infinigen_common::blocks::{BlockColor, BlockID, BlockType, BlockVisibility, Palette};
use infinigen_common::mesh::faces::BlockVisibilityChecker;
use infinigen_common::mesh::textures::{BlockAppearances, Face};
use infinigen_common::world::MappedBlockID;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, EnumCount)]
pub enum MaterialType {
    DenseOpaque = 0,
    Translucent,
}

type TextureFilename = String;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, TypePath, Asset,
)]
pub struct BlockDefinition {
    pub id: BlockID,
    #[serde(default)]
    pub visibility: BlockVisibility,
    #[serde(default = "default_block_color")]
    pub color: BlockColor,
    pub textures: Option<BTreeMap<Face, TextureFilename>>,
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

impl From<&BlockType> for BlockDefinition {
    fn from(value: &BlockType) -> Self {
        Self {
            color: value.color,
            id: value.id.clone(),
            visibility: value.visibility,
            textures: Default::default(),
        }
    }
}

fn default_block_color() -> BlockColor {
    [255, 255, 255, 255]
}

/// Tracks which [`MappedBlockID`]s are being used for which [`BlockDefinition`]s for the currently loaded session.
#[derive(Debug, Default, Clone)]
pub struct BlockDefinitions {
    by_mapped_id: AHashMap<MappedBlockID, BlockDefinition>,
    next_free_mapped_id: MappedBlockID,
}

impl BlockDefinitions {
    pub fn get(&self, mapped_id: &MappedBlockID) -> &BlockDefinition {
        self.by_mapped_id.get(mapped_id).unwrap()
    }

    /// Adds a block definition to the mappings and returns the mapped ID, or None if no more IDs are available.
    pub fn add(&mut self, block_definition: BlockDefinition) -> Option<MappedBlockID> {
        let mapped_id = self.next_free_mapped_id;
        tracing::debug!(?block_definition, ?mapped_id, "Mapping block");
        self.by_mapped_id
            .insert(self.next_free_mapped_id, block_definition);
        self.next_free_mapped_id = self.next_free_mapped_id.next()?;
        Some(mapped_id)
    }

    pub fn palette(&self) -> Palette {
        let mut palette = Palette::default();
        for (mapped_id, block_definition) in self.by_mapped_id.iter() {
            palette
                .inner
                .insert(block_definition.id.clone(), *mapped_id);
        }
        palette
    }
}

impl BlockVisibilityChecker for BlockDefinitions {
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility {
        self.get(mapped_id).visibility
    }
}

#[derive(Default, Resource)]
pub struct BlockRegistry {
    pub materials: [Handle<StandardMaterial>; MaterialType::COUNT],
    pub appearances: BlockAppearances,
    pub definitions: BlockDefinitions,
}

impl BlockRegistry {
    /// Returns a weak handle to a material.
    pub fn get_material(&self, material_type: MaterialType) -> Handle<StandardMaterial> {
        self.materials[material_type as usize].clone_weak()
    }

    pub fn get_appearances(&self) -> &BlockAppearances {
        &self.appearances
    }
}
