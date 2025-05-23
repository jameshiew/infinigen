use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

use crate::world::MappedBlockID;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VoxelBlock {
    Opaque(MappedBlockID),
    Translucent(MappedBlockID),
    Empty,
}

impl Voxel for VoxelBlock {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Self::Opaque(_) => VoxelVisibility::Opaque,
            Self::Translucent(_) => VoxelVisibility::Translucent,
            Self::Empty => VoxelVisibility::Empty,
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
