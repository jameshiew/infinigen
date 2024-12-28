use std::sync::Arc;

use ahash::AHashMap;
use bevy::prelude::Resource;
use infinigen_common::blocks::BlockVisibility;
use infinigen_common::chunks::{Array3Chunk, Chunk, CHUNK_SIZE};
use infinigen_common::mesh::faces::{extract_faces, BlockVisibilityChecker};
use infinigen_common::mesh::shapes::{ChunkFace, EMPTY_CHUNK_FACES};
use infinigen_common::world::{BlockPosition, ChunkPosition, Direction, MappedBlockID};
use infinigen_common::zoom::ZoomLevel;
use linearize::StaticCopyMap;
use strum::IntoEnumIterator;

use crate::world::World;

#[derive(Default, Resource)]
pub struct ChunkRegistry {
    chunks: AHashMap<ZoomLevel, AHashMap<ChunkPosition, ChunkStatus>>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub opaque: Box<Array3Chunk>,
    pub translucents: Vec<Array3Chunk>,
    pub faces: StaticCopyMap<Direction, ChunkFace>,
}

#[derive(Debug, Clone)]
pub enum ChunkStatus {
    Requested,
    Generated(Arc<ChunkInfo>),
    Empty,
}

impl ChunkRegistry {
    pub fn get_status(&self, zoom_level: ZoomLevel, pos: &ChunkPosition) -> Option<&ChunkStatus> {
        self.chunks
            .get(&zoom_level)
            .and_then(|chunks| chunks.get(pos))
    }

    pub fn set_status(&mut self, zoom_level: ZoomLevel, pos: &ChunkPosition, status: ChunkStatus) {
        self.chunks
            .entry(zoom_level)
            .or_default()
            .insert(*pos, status);
    }

    pub fn insert_generated(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        chunk: Chunk,
        visibility_checker: impl BlockVisibilityChecker,
    ) {
        let status = match chunk {
            Chunk::Empty => ChunkStatus::Empty,
            Chunk::Array3(mut chunk) => {
                let faces = extract_faces(chunk.as_ref(), &visibility_checker);
                let translucents = split_out_translucent(&mut chunk, &visibility_checker);
                ChunkStatus::Generated(Arc::new(ChunkInfo {
                    opaque: chunk,
                    translucents: translucents.into_values().collect(),
                    faces,
                }))
            }
        };
        self.set_status(zoom_level, position, status);
    }

    fn get_faces_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        visibility_checker: impl BlockVisibilityChecker,
    ) -> StaticCopyMap<Direction, ChunkFace> {
        match self.get_status(zoom_level, position) {
            Some(ChunkStatus::Generated(chunk_info)) => chunk_info.faces,
            Some(ChunkStatus::Empty) => EMPTY_CHUNK_FACES,
            _ => {
                let chunk = world.get_chunk(zoom_level, position);
                self.insert_generated(zoom_level, position, chunk, visibility_checker);
                match self.get_status(zoom_level, position).unwrap() {
                    ChunkStatus::Generated(chunk_info) => chunk_info.faces,
                    ChunkStatus::Empty => EMPTY_CHUNK_FACES,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn get_neighboring_faces_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        visibility_checker: impl BlockVisibilityChecker,
    ) -> StaticCopyMap<Direction, ChunkFace> {
        let mut neighbor_faces = EMPTY_CHUNK_FACES;
        for dir in Direction::iter() {
            let normal: [i32; 3] = dir.into();
            let neighbor_pos = ChunkPosition {
                x: position.x + normal[0],
                y: position.y + normal[1],
                z: position.z + normal[2],
            };
            let faces = self.get_faces_mut(zoom_level, &neighbor_pos, world, &visibility_checker);
            let opposite = dir.opposite();
            neighbor_faces[dir] = faces[opposite];
        }
        neighbor_faces
    }
}

pub fn get_neighbour_cposes(
    position: &ChunkPosition,
) -> impl Iterator<Item = (Direction, ChunkPosition)> + '_ {
    Direction::iter().map(|dir| {
        let normal: [i32; 3] = dir.into();
        (
            dir,
            ChunkPosition {
                x: position.x + normal[0],
                y: position.y + normal[1],
                z: position.z + normal[2],
            },
        )
    })
}

/// Splits out translucent chunks from chunk, leaving only opaque blocks.
pub fn split_out_translucent(
    chunk: &mut Array3Chunk,
    visibility_checker: impl BlockVisibilityChecker,
) -> AHashMap<MappedBlockID, Array3Chunk> {
    let mut translucents = AHashMap::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if let Some(block_id) = chunk.get(&BlockPosition { x, y, z }) {
                    if visibility_checker.get_visibility(&block_id) == BlockVisibility::Opaque {
                        continue;
                    }
                    translucents
                        .entry(block_id)
                        .or_insert_with(Array3Chunk::default)
                        .insert(&BlockPosition { x, y, z }, block_id);
                    chunk.clear(&BlockPosition { x, y, z });
                }
            }
        }
    }

    translucents
}
