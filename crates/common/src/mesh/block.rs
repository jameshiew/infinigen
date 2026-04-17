use crate::world::MappedBlockID;

/// Visibility category used to decide whether a face between two voxels
/// should be meshed.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VoxelVisibility {
    Empty,
    Translucent,
    Opaque,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VoxelBlock {
    Opaque(MappedBlockID),
    Translucent(MappedBlockID),
    Empty,
}

impl VoxelBlock {
    #[inline]
    pub const fn visibility(self) -> VoxelVisibility {
        match self {
            Self::Opaque(_) => VoxelVisibility::Opaque,
            Self::Translucent(_) => VoxelVisibility::Translucent,
            Self::Empty => VoxelVisibility::Empty,
        }
    }
}
