use block_mesh::ndshape::ConstShape;
use block_mesh::{OrientedBlockFace, RIGHT_HANDED_Y_UP_CONFIG};
use linearize::StaticCopyMap;
use strum::IntoEnumIterator;

use super::shapes::EMPTY_CHUNK_FACES;
use crate::blocks::BlockVisibility;
use crate::chunks::{Array3Chunk, CHUNK_SIZE, CHUNK_SIZE_U32};
use crate::mesh::block::VoxelBlock;
use crate::mesh::shapes::{ChunkFace, ChunkFaceShape, PaddedChunk, PaddedChunkShape};
use crate::world::{BlockPosition, Direction, MappedBlockID};

pub const RHS_FACES: [OrientedBlockFace; 6] = RIGHT_HANDED_Y_UP_CONFIG.faces;

const PADDED_BOUNDARY_MIN: u32 = 0;
const PADDED_BOUNDARY_MAX: u32 = CHUNK_SIZE_U32 + 1;

// Encodes the orientation of a face within the chunk and its matching padded boundary.
#[derive(Clone, Copy)]
struct FaceSpec {
    fixed_axis: usize,
    fixed_value: u8,
    iter_axes: [usize; 2],
    padded_boundary_index: u32,
}

const fn face_spec(direction: Direction) -> FaceSpec {
    match direction {
        Direction::Up => FaceSpec {
            fixed_axis: 1,
            fixed_value: BlockPosition::MAX_IDX,
            iter_axes: [0, 2],
            padded_boundary_index: PADDED_BOUNDARY_MAX,
        },
        Direction::Down => FaceSpec {
            fixed_axis: 1,
            fixed_value: BlockPosition::MIN_IDX,
            iter_axes: [0, 2],
            padded_boundary_index: PADDED_BOUNDARY_MIN,
        },
        Direction::North => FaceSpec {
            fixed_axis: 2,
            fixed_value: BlockPosition::MIN_IDX,
            iter_axes: [0, 1],
            padded_boundary_index: PADDED_BOUNDARY_MIN,
        },
        Direction::South => FaceSpec {
            fixed_axis: 2,
            fixed_value: BlockPosition::MAX_IDX,
            iter_axes: [0, 1],
            padded_boundary_index: PADDED_BOUNDARY_MAX,
        },
        Direction::East => FaceSpec {
            fixed_axis: 0,
            fixed_value: BlockPosition::MAX_IDX,
            iter_axes: [1, 2],
            padded_boundary_index: PADDED_BOUNDARY_MAX,
        },
        Direction::West => FaceSpec {
            fixed_axis: 0,
            fixed_value: BlockPosition::MIN_IDX,
            iter_axes: [1, 2],
            padded_boundary_index: PADDED_BOUNDARY_MIN,
        },
    }
}

fn chunk_face_index(u: u8, v: u8) -> usize {
    ChunkFaceShape::linearize([u as u32, v as u32]) as usize
}

const fn face_block_position(spec: FaceSpec, u: u8, v: u8) -> BlockPosition {
    let mut coords = [0u8; 3];
    coords[spec.iter_axes[0]] = u;
    coords[spec.iter_axes[1]] = v;
    coords[spec.fixed_axis] = spec.fixed_value;
    BlockPosition {
        x: coords[0],
        y: coords[1],
        z: coords[2],
    }
}

const fn padded_coords(spec: FaceSpec, u: u8, v: u8) -> [u32; 3] {
    let mut coords = [1u32; 3];
    coords[spec.iter_axes[0]] = u as u32 + 1;
    coords[spec.iter_axes[1]] = v as u32 + 1;
    coords[spec.fixed_axis] = spec.padded_boundary_index;
    coords
}

fn map_block(block: Option<MappedBlockID>, checker: &impl BlockVisibilityChecker) -> VoxelBlock {
    block.map_or(VoxelBlock::Empty, |mapped_id| {
        map_non_empty(mapped_id, checker)
    })
}

fn map_non_empty(mapped_id: MappedBlockID, checker: &impl BlockVisibilityChecker) -> VoxelBlock {
    match checker.get_visibility(&mapped_id) {
        BlockVisibility::Opaque => VoxelBlock::Opaque(mapped_id),
        BlockVisibility::Translucent => VoxelBlock::Translucent(mapped_id),
    }
}

pub trait BlockVisibilityChecker: Clone + Send + Sync {
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility;
}

impl<T> BlockVisibilityChecker for &T
where
    T: BlockVisibilityChecker,
{
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility {
        (*self).get_visibility(mapped_id)
    }
}

pub fn extract_faces(
    chunk: &Array3Chunk,
    visibility_checker: impl BlockVisibilityChecker,
) -> StaticCopyMap<Direction, ChunkFace> {
    let mut faces = EMPTY_CHUNK_FACES;
    let checker = &visibility_checker;
    for dir in Direction::iter() {
        let spec = face_spec(dir);
        for u in 0..CHUNK_SIZE {
            for v in 0..CHUNK_SIZE {
                let position = face_block_position(spec, u, v);
                let index = chunk_face_index(u, v);
                let block = chunk.get(&position);
                faces[dir][index] = map_block(block, checker);
            }
        }
    }
    faces
}

/// Puts `chunk` into a `ChunkShape` with 1-voxel padding around the edges, filled according to the neighbor faces.
pub fn prepare_padded_chunk(
    chunk: &Array3Chunk,
    neighbor_faces: &StaticCopyMap<Direction, ChunkFace>,
    visibility_checker: impl BlockVisibilityChecker,
) -> PaddedChunk {
    let mut padded = [VoxelBlock::Empty; PaddedChunkShape::SIZE as usize];
    let checker = &visibility_checker;

    for dir in Direction::iter() {
        let spec = face_spec(dir);
        let neighboring_face = neighbor_faces[dir];
        for u in 0..CHUNK_SIZE {
            for v in 0..CHUNK_SIZE {
                let face_index = chunk_face_index(u, v);
                let padded_index = PaddedChunkShape::linearize(padded_coords(spec, u, v)) as usize;
                padded[padded_index] = neighboring_face[face_index];
            }
        }
    }

    for bx in 0..CHUNK_SIZE {
        for by in 0..CHUNK_SIZE {
            for bz in 0..CHUNK_SIZE {
                let block = chunk.get(&BlockPosition {
                    x: bx,
                    y: by,
                    z: bz,
                });
                if let Some(mapped_id) = block {
                    let i =
                        PaddedChunkShape::linearize([bx as u32 + 1, by as u32 + 1, bz as u32 + 1]);
                    padded[i as usize] = map_non_empty(mapped_id, checker);
                }
            }
        }
    }
    padded
}

#[cfg(test)]
mod tests {
    use block_mesh::{UnitQuadBuffer, visible_block_faces};
    use linearize::static_copy_map;

    use crate::blocks::BlockVisibility;
    use crate::chunks::{CHUNK_SIZE_U32, filled_chunk};
    use crate::mesh::shapes::{
        ChunkFace, ChunkFaceShape, PADDED_CHUNK_MAX_INDEX, PADDED_CHUNK_SIZE,
    };

    #[derive(Clone)]
    struct AllOpaque;

    impl BlockVisibilityChecker for AllOpaque {
        fn get_visibility(&self, _mapped_id: &MappedBlockID) -> BlockVisibility {
            BlockVisibility::Opaque
        }
    }

    use super::*;

    const PADDED_CHUNK_MIN_INDEX: u32 = 0;

    pub fn full_chunk_face() -> ChunkFace {
        [VoxelBlock::Opaque(MappedBlockID::default()); ChunkFaceShape::SIZE as usize]
    }

    #[test]
    fn test_prepare_padded_chunk_with_empty_faces() {
        let full = filled_chunk(MappedBlockID::default());
        let empties = EMPTY_CHUNK_FACES;
        let padded = prepare_padded_chunk(&full, &empties, AllOpaque);
        // faces of padded chunk remain empty
        for x in 0..CHUNK_SIZE_U32 + 2 {
            for y in 0..CHUNK_SIZE_U32 + 2 {
                for z in 0..CHUNK_SIZE_U32 + 2 {
                    let i = PaddedChunkShape::linearize([x, y, z]);
                    if x == 0
                        || x == CHUNK_SIZE_U32 + 1
                        || y == 0
                        || y == CHUNK_SIZE_U32 + 1
                        || z == 0
                        || z == CHUNK_SIZE_U32 + 1
                    {
                        assert!(matches!(padded[i as usize], VoxelBlock::Empty))
                    } else {
                        assert!(matches!(padded[i as usize], VoxelBlock::Opaque(_)))
                    }
                }
            }
        }

        let mut buffer = UnitQuadBuffer::new();
        visible_block_faces(
            &padded,
            &PaddedChunkShape {},
            [0; 3],
            [PADDED_CHUNK_MAX_INDEX; 3],
            &RHS_FACES,
            &mut buffer,
        );
        assert_eq!(buffer.groups.len(), 6);
        let mut quads = 0;
        for group in buffer.groups {
            for _ in group {
                quads += 1;
            }
        }
        assert_eq!(
            quads,
            6 * CHUNK_SIZE_U32 * CHUNK_SIZE_U32,
            "all blocks on the faces of the chunk are exposed, but none within"
        );
    }

    #[test]
    fn test_prepare_padded_chunk_with_full_faces() {
        let full = filled_chunk(MappedBlockID::default());
        let full_chunk_faces = static_copy_map! {
            Direction::Up => full_chunk_face(),
            Direction::Down => full_chunk_face(),
            Direction::North => full_chunk_face(),
            Direction::South => full_chunk_face(),
            Direction::East => full_chunk_face(),
            Direction::West => full_chunk_face()
        };
        let padded = prepare_padded_chunk(&full, &full_chunk_faces, AllOpaque);

        // faces of padded chunk are all full, except for corners
        let mut empty: u32 = 0;
        let mut full: u32 = 0;
        for x in 0..PADDED_CHUNK_SIZE {
            for y in 0..PADDED_CHUNK_SIZE {
                for z in 0..PADDED_CHUNK_SIZE {
                    let i = PaddedChunkShape::linearize([x, y, z]);
                    if matches!(padded[i as usize], VoxelBlock::Empty) {
                        empty += 1;
                    } else {
                        full += 1;
                    }

                    let mut on_n_outer_faces = 0;
                    let on_x_outer_face =
                        x == PADDED_CHUNK_MIN_INDEX || x == PADDED_CHUNK_MAX_INDEX;
                    if on_x_outer_face {
                        on_n_outer_faces += 1;
                    }
                    let on_y_outer_face =
                        y == PADDED_CHUNK_MIN_INDEX || y == PADDED_CHUNK_MAX_INDEX;
                    if on_y_outer_face {
                        on_n_outer_faces += 1;
                    }
                    let on_z_outer_face =
                        z == PADDED_CHUNK_MIN_INDEX || z == PADDED_CHUNK_MAX_INDEX;
                    if on_z_outer_face {
                        on_n_outer_faces += 1;
                    }

                    if on_n_outer_faces >= 2 {
                        assert!(matches!(padded[i as usize], VoxelBlock::Empty))
                    } else {
                        assert!(
                            matches!(padded[i as usize], VoxelBlock::Opaque(_)),
                            "at {:?}",
                            [x, y, z]
                        )
                    }
                }
            }
        }
        println!("empty: {empty}, full: {full}");
        // edges are empty
        assert_eq!(
            empty,
            ((PADDED_CHUNK_SIZE * 4) - 4) * 2 + (PADDED_CHUNK_SIZE - 2) * 4
        );
        // faces + inner chunk are full
        assert_eq!(
            full,
            (CHUNK_SIZE_U32 * CHUNK_SIZE_U32 * 6)
                + (CHUNK_SIZE_U32 * CHUNK_SIZE_U32 * CHUNK_SIZE_U32)
        );

        let mut buffer = UnitQuadBuffer::new();
        visible_block_faces(
            &padded,
            &PaddedChunkShape {},
            [0; 3],
            [PADDED_CHUNK_MAX_INDEX; 3],
            &RHS_FACES,
            &mut buffer,
        );
        assert_eq!(buffer.groups.len(), 6);
        let mut quads = 0;
        for group in buffer.groups {
            for _ in group {
                quads += 1;
            }
        }
        assert_eq!(
            quads, 0,
            "all blocks on the faces of the chunk are occluded by neighboring chunks"
        );
    }
}
