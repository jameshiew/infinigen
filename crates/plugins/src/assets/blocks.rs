use ahash::{AHashMap, AHashSet};
use bevy::prelude::*;
use infinigen_common::blocks::{BlockType, BlockVisibility, Palette};
use infinigen_common::mesh::faces::BlockVisibilityChecker;
use infinigen_common::mesh::textures::BlockAppearances;
use infinigen_common::world::MappedBlockID;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

#[derive(
    Default,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    TypePath,
    Asset,
)]
#[serde(transparent)]
pub struct BlockDefinition(pub BlockType);

impl From<BlockType> for BlockDefinition {
    fn from(value: BlockType) -> Self {
        BlockDefinition(value)
    }
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
                .insert(block_definition.0.id.clone(), *mapped_id);
        }
        palette
    }

    pub fn visibility_checker(&self) -> impl BlockVisibilityChecker {
        #[derive(Clone)]
        struct Checker {
            opaque: AHashSet<MappedBlockID>,
        }
        impl BlockVisibilityChecker for Checker {
            fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility {
                if self.opaque.contains(mapped_id) {
                    BlockVisibility::Opaque
                } else {
                    BlockVisibility::Translucent
                }
            }
        }
        Checker {
            opaque: self
                .by_mapped_id
                .iter()
                .filter_map(|(id, def)| {
                    if def.0.visibility == BlockVisibility::Opaque {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

impl BlockVisibilityChecker for BlockDefinitions {
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility {
        self.get(mapped_id).0.visibility
    }
}

#[derive(Default, Resource)]
pub struct BlockRegistry {
    pub materials: [Handle<StandardMaterial>; BlockVisibility::COUNT],
    pub appearances: BlockAppearances,
    pub definitions: BlockDefinitions,
}

impl BlockRegistry {
    pub fn get_material(&self, visibility: BlockVisibility) -> Handle<StandardMaterial> {
        self.materials[visibility as usize].clone_weak()
    }
}
