use block_mesh::ndshape::ConstShape;
use block_mesh::{OrientedBlockFace, RIGHT_HANDED_Y_UP_CONFIG};
use strum::IntoEnumIterator;

use crate::blocks::BlockVisibility;
use crate::chunks::{Array3Chunk, CHUNK_SIZE, CHUNK_SIZE_U32};
use crate::mesh::block::VoxelBlock;
use crate::mesh::shapes;
use crate::mesh::shapes::{ChunkFace, ChunkFaceShape, PaddedChunk, PaddedChunkShape};
use crate::world::{BlockPosition, MappedBlockID};

pub const RHS_FACES: [OrientedBlockFace; 6] = RIGHT_HANDED_Y_UP_CONFIG.faces;

pub trait BlockVisibilityChecker {
    fn get_visibility(&self, mapped_id: &MappedBlockID) -> BlockVisibility;
}

pub fn extract_faces(
    chunk: &Array3Chunk,
    block_mappings: impl BlockVisibilityChecker,
) -> [ChunkFace; 6] {
    let mut faces = [shapes::EMPTY_CHUNK_FACE; 6];
    for dir in crate::world::Direction::iter() {
        match dir {
            crate::world::Direction::Up => {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = chunk.get(&BlockPosition {
                            x,
                            y: BlockPosition::MAX_IDX,
                            z,
                        });
                        let j = ChunkFaceShape::linearize([x as u32, z as u32]) as usize;
                        let block = match block {
                            Some(chunk_block_id) => {
                                match block_mappings.get_visibility(&chunk_block_id) {
                                    crate::blocks::BlockVisibility::Opaque => {
                                        VoxelBlock::Opaque(chunk_block_id)
                                    }
                                    crate::blocks::BlockVisibility::Translucent => {
                                        VoxelBlock::Translucent(chunk_block_id)
                                    }
                                }
                            }
                            None => VoxelBlock::Empty,
                        };
                        faces[dir as usize][j] = block;
                    }
                }
            }
            crate::world::Direction::Down => {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = chunk.get(&BlockPosition {
                            x,
                            y: BlockPosition::MIN_IDX,
                            z,
                        });
                        let j = ChunkFaceShape::linearize([x as u32, z as u32]) as usize;
                        let block = match block {
                            Some(chunk_block_id) => {
                                match block_mappings.get_visibility(&chunk_block_id) {
                                    crate::blocks::BlockVisibility::Opaque => {
                                        VoxelBlock::Opaque(chunk_block_id)
                                    }
                                    crate::blocks::BlockVisibility::Translucent => {
                                        VoxelBlock::Translucent(chunk_block_id)
                                    }
                                }
                            }
                            None => VoxelBlock::Empty,
                        };
                        faces[dir as usize][j] = block;
                    }
                }
            }
            crate::world::Direction::North => {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        let block = chunk.get(&BlockPosition {
                            x,
                            y,
                            z: BlockPosition::MIN_IDX,
                        });
                        let j = ChunkFaceShape::linearize([x as u32, y as u32]) as usize;
                        let block = match block {
                            Some(chunk_block_id) => {
                                match block_mappings.get_visibility(&chunk_block_id) {
                                    crate::blocks::BlockVisibility::Opaque => {
                                        VoxelBlock::Opaque(chunk_block_id)
                                    }
                                    crate::blocks::BlockVisibility::Translucent => {
                                        VoxelBlock::Translucent(chunk_block_id)
                                    }
                                }
                            }
                            None => VoxelBlock::Empty,
                        };
                        faces[dir as usize][j] = block;
                    }
                }
            }
            crate::world::Direction::South => {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        let block = chunk.get(&BlockPosition {
                            x,
                            y,
                            z: BlockPosition::MAX_IDX,
                        });
                        let j = ChunkFaceShape::linearize([x as u32, y as u32]) as usize;
                        let block = match block {
                            Some(chunk_block_id) => {
                                match block_mappings.get_visibility(&chunk_block_id) {
                                    crate::blocks::BlockVisibility::Opaque => {
                                        VoxelBlock::Opaque(chunk_block_id)
                                    }
                                    crate::blocks::BlockVisibility::Translucent => {
                                        VoxelBlock::Translucent(chunk_block_id)
                                    }
                                }
                            }
                            None => VoxelBlock::Empty,
                        };
                        faces[dir as usize][j] = block;
                    }
                }
            }
            crate::world::Direction::East => {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = chunk.get(&BlockPosition {
                            x: BlockPosition::MAX_IDX,
                            y,
                            z,
                        });
                        let j = ChunkFaceShape::linearize([y as u32, z as u32]) as usize;
                        let block = match block {
                            Some(chunk_block_id) => {
                                match block_mappings.get_visibility(&chunk_block_id) {
                                    crate::blocks::BlockVisibility::Opaque => {
                                        VoxelBlock::Opaque(chunk_block_id)
                                    }
                                    crate::blocks::BlockVisibility::Translucent => {
                                        VoxelBlock::Translucent(chunk_block_id)
                                    }
                                }
                            }
                            None => VoxelBlock::Empty,
                        };
                        faces[dir as usize][j] = block;
                    }
                }
            }
            crate::world::Direction::West => {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = chunk.get(&BlockPosition {
                            x: BlockPosition::MIN_IDX,
                            y,
                            z,
                        });
                        let j = ChunkFaceShape::linearize([y as u32, z as u32]) as usize;
                        let block = match block {
                            Some(chunk_block_id) => {
                                match block_mappings.get_visibility(&chunk_block_id) {
                                    crate::blocks::BlockVisibility::Opaque => {
                                        VoxelBlock::Opaque(chunk_block_id)
                                    }
                                    crate::blocks::BlockVisibility::Translucent => {
                                        VoxelBlock::Translucent(chunk_block_id)
                                    }
                                }
                            }
                            None => VoxelBlock::Empty,
                        };
                        faces[dir as usize][j] = block;
                    }
                }
            }
        }
    }
    faces
}

/// Puts `chunk` into a `ChunkShape` with 1-voxel padding around the edges, filled according to the neighbor faces.
pub fn prepare_padded_chunk(
    chunk: &Array3Chunk,
    neighbor_faces: &[ChunkFace; 6],
    block_mappings: impl BlockVisibilityChecker,
) -> PaddedChunk {
    let mut padded = [VoxelBlock::Empty; PaddedChunkShape::SIZE as usize];
    const MIN_PADDED_IDX: u32 = 0;
    const MAX_PADDED_IDX: u32 = CHUNK_SIZE_U32 + 1;

    for dir in crate::world::Direction::iter() {
        let neighboring_face = neighbor_faces[dir as usize];
        match dir {
            // bottom face of above chunk
            crate::world::Direction::Up => {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = neighboring_face
                            [ChunkFaceShape::linearize([x as u32, z as u32]) as usize];
                        let i = PaddedChunkShape::linearize([
                            x as u32 + 1,
                            MAX_PADDED_IDX,
                            z as u32 + 1,
                        ]);
                        padded[i as usize] = block;
                    }
                }
            }
            // top face of below chunk
            crate::world::Direction::Down => {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = neighboring_face
                            [ChunkFaceShape::linearize([x as u32, z as u32]) as usize];
                        let i = PaddedChunkShape::linearize([
                            x as u32 + 1,
                            MIN_PADDED_IDX,
                            z as u32 + 1,
                        ]);
                        padded[i as usize] = block;
                    }
                }
            }
            // south face of chunk to the north, etc.
            crate::world::Direction::North => {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        let block = neighboring_face
                            [ChunkFaceShape::linearize([x as u32, y as u32]) as usize];
                        let i = PaddedChunkShape::linearize([
                            x as u32 + 1,
                            y as u32 + 1,
                            MIN_PADDED_IDX,
                        ]);
                        padded[i as usize] = block;
                    }
                }
            }
            crate::world::Direction::South => {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        let block = neighboring_face
                            [ChunkFaceShape::linearize([x as u32, y as u32]) as usize];
                        let i = PaddedChunkShape::linearize([
                            x as u32 + 1,
                            y as u32 + 1,
                            MAX_PADDED_IDX,
                        ]);
                        padded[i as usize] = block;
                    }
                }
            }
            crate::world::Direction::East => {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = neighboring_face
                            [ChunkFaceShape::linearize([y as u32, z as u32]) as usize];
                        let i = PaddedChunkShape::linearize([
                            MAX_PADDED_IDX,
                            y as u32 + 1,
                            z as u32 + 1,
                        ]);
                        padded[i as usize] = block;
                    }
                }
            }
            crate::world::Direction::West => {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let block = neighboring_face
                            [ChunkFaceShape::linearize([y as u32, z as u32]) as usize];
                        let i = PaddedChunkShape::linearize([
                            MIN_PADDED_IDX,
                            y as u32 + 1,
                            z as u32 + 1,
                        ]);
                        padded[i as usize] = block;
                    }
                }
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
                if block.is_some() {
                    let i =
                        PaddedChunkShape::linearize([bx as u32 + 1, by as u32 + 1, bz as u32 + 1]);
                    let block = match block {
                        Some(chunk_block_id) => {
                            match block_mappings.get_visibility(&chunk_block_id) {
                                crate::blocks::BlockVisibility::Opaque => {
                                    VoxelBlock::Opaque(chunk_block_id)
                                }
                                crate::blocks::BlockVisibility::Translucent => {
                                    VoxelBlock::Translucent(chunk_block_id)
                                }
                            }
                        }
                        None => VoxelBlock::Empty,
                    };
                    padded[i as usize] = block;
                }
            }
        }
    }
    padded
}

#[cfg(test)]
mod tests {
    use block_mesh::{visible_block_faces, UnitQuadBuffer};
    use shapes::EMPTY_CHUNK_FACES;

    use crate::blocks::BlockVisibility;
    use crate::chunks::{filled_chunk, CHUNK_SIZE_U32};
    use crate::mesh::shapes::{
        ChunkFace, ChunkFaceShape, PADDED_CHUNK_MAX_INDEX, PADDED_CHUNK_SIZE,
    };

    struct AllOpaque;

    impl BlockVisibilityChecker for AllOpaque {
        fn get_visibility(&self, _mapped_id: &MappedBlockID) -> BlockVisibility {
            BlockVisibility::Opaque
        }
    }

    use super::*;

    const PADDED_CHUNK_MIN_INDEX: u32 = 0;

    pub fn full_chunk_face() -> ChunkFace {
        [VoxelBlock::Opaque(0); ChunkFaceShape::SIZE as usize]
    }

    #[test]
    fn test_prepare_padded_chunk_with_empty_faces() {
        let full = filled_chunk(0);
        let padded = prepare_padded_chunk(&full, &EMPTY_CHUNK_FACES, AllOpaque);
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
        let full = filled_chunk(0);
        let padded = prepare_padded_chunk(&full, &[full_chunk_face(); 6], AllOpaque);

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
        println!("empty: {}, full: {}", empty, full);
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
