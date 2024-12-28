use ahash::AHashMap;
use bevy::prelude::Resource;
use infinigen_common::chunks::{Array3Chunk, Chunk};
use infinigen_common::mesh::faces::extract_faces;
use infinigen_common::mesh::shapes::{ChunkFace, EMPTY_CHUNK_FACES};
use infinigen_common::world::{ChunkPosition, Direction};
use infinigen_common::zoom::ZoomLevel;
use linearize::StaticCopyMap;
use strum::IntoEnumIterator;

use crate::assets::blocks::BlockMappings;
use crate::world::World;

#[derive(Default, Resource)]
pub struct ChunkRegistry {
    chunks: AHashMap<ZoomLevel, AHashMap<ChunkPosition, ChunkStatus>>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub chunk: Box<Array3Chunk>,
    pub faces: StaticCopyMap<Direction, ChunkFace>,
}

#[derive(Debug, Clone)]
pub enum ChunkStatus {
    Requested,
    Generated(Box<ChunkInfo>),
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
        block_mappings: &BlockMappings,
    ) {
        let status = match chunk {
            Chunk::Empty => ChunkStatus::Empty,
            Chunk::Array3(ref array3_chunk) => ChunkStatus::Generated(Box::new(ChunkInfo {
                chunk: array3_chunk.clone(),
                faces: extract_faces(array3_chunk.as_ref(), block_mappings),
            })),
        };
        self.set_status(zoom_level, position, status);
    }

    fn get_faces_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        block_mappings: &BlockMappings,
    ) -> StaticCopyMap<Direction, ChunkFace> {
        match self.get_status(zoom_level, position) {
            Some(ChunkStatus::Generated(chunk_info)) => chunk_info.faces,
            Some(ChunkStatus::Empty) => EMPTY_CHUNK_FACES,
            _ => {
                let chunk = world.get_chunk(zoom_level, position);
                self.insert_generated(zoom_level, position, chunk, block_mappings);
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
        block_mappings: &BlockMappings,
    ) -> StaticCopyMap<Direction, ChunkFace> {
        let mut neighbor_faces = EMPTY_CHUNK_FACES;
        for dir in Direction::iter() {
            let normal: [i32; 3] = dir.into();
            let neighbor_pos = ChunkPosition {
                x: position.x + normal[0],
                y: position.y + normal[1],
                z: position.z + normal[2],
            };
            let faces = self.get_faces_mut(zoom_level, &neighbor_pos, world, block_mappings);
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
