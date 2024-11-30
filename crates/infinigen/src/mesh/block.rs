//! This module is concerned with meshing chunks, and not with rendering.
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

use crate::common::world::ChunkBlockId;

/// Wraps the common block type into something that we can implement block mesh traits for e.g. [`MergeVoxel`] and [`Voxel`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VoxelBlock {
    Opaque(ChunkBlockId),
    Translucent(ChunkBlockId),
    Empty,
}

impl VoxelBlock {
    pub fn new(block: ChunkBlockId) -> Self {
        Self::Opaque(block)
    }
}

impl Voxel for VoxelBlock {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            VoxelBlock::Opaque(_) => VoxelVisibility::Opaque,
            VoxelBlock::Translucent(_) => VoxelVisibility::Translucent,
            VoxelBlock::Empty => VoxelVisibility::Empty,
        }
    }
}

impl MergeVoxel for VoxelBlock {
    type MergeValue = u8;
    type MergeValueFacingNeighbour = u8;

    fn merge_value(&self) -> Self::MergeValue {
        0
    }

    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
        0
    }
}
