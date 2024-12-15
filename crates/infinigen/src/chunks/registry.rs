use ahash::AHashMap;
use bevy::prelude::Resource;
use infinigen_common::chunks::{Array3Chunk, Chunk};
use infinigen_common::mesh::faces::extract_faces;
use infinigen_common::mesh::shapes::{ChunkFace, EMPTY_CHUNK_FACES};
use infinigen_common::world::{ChunkPosition, Direction};
use infinigen_common::zoom::ZoomLevel;
use strum::IntoEnumIterator;

use crate::assets::blocks::BlockMappings;
use crate::world::World;

// Responsible for keeping track of chunks.
#[derive(Default, Resource)]
pub struct ChunkRegistry {
    cached: AHashMap<ZoomLevel, AHashMap<ChunkPosition, ChunkStatus>>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub chunk: Box<Array3Chunk>,
    pub faces: [ChunkFace; 6],
}

#[derive(Debug, Clone)]
pub enum ChunkStatus {
    Requested,
    Generated(Box<ChunkInfo>),
    Empty,
}

impl ChunkRegistry {
    pub fn clear(&mut self) {
        self.cached.clear();
    }

    pub fn get_status(&self, zoom_level: ZoomLevel, cpos: &ChunkPosition) -> Option<&ChunkStatus> {
        match self.cached.get(&zoom_level) {
            Some(chunks) => chunks.get(cpos),
            None => None,
        }
    }

    pub fn set_status(&mut self, zoom_level: ZoomLevel, cpos: &ChunkPosition, status: ChunkStatus) {
        match self.cached.get_mut(&zoom_level) {
            Some(chunks) => {
                chunks.insert(*cpos, status);
            }
            None => {
                let mut chunks = AHashMap::default();
                chunks.insert(*cpos, status);
                self.cached.insert(zoom_level, chunks);
            }
        }
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
    ) -> [ChunkFace; 6] {
        match self.get_status(zoom_level, position) {
            Some(ChunkStatus::Generated(chunk_info)) => return chunk_info.faces,
            Some(ChunkStatus::Empty) => return EMPTY_CHUNK_FACES,
            _ => {}
        }
        let chunk = world.get_chunk(zoom_level, position);
        self.insert_generated(zoom_level, position, chunk, block_mappings);
        match self.get_status(zoom_level, position).unwrap() {
            ChunkStatus::Generated(chunk_info) => chunk_info.faces,
            ChunkStatus::Empty => EMPTY_CHUNK_FACES,
            _ => unreachable!(),
        }
    }

    /// Returns the faces of neighboring chunks, in direction order. The bottom face of the chunk above, then the top face of the chunk below, etc.
    pub fn get_neighboring_faces_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        block_mappings: &BlockMappings,
    ) -> [ChunkFace; 6] {
        let mut neighbor_faces = EMPTY_CHUNK_FACES;
        for dir in infinigen_common::world::Direction::iter() {
            let normal: [i32; 3] = dir.into();
            let neighbor_cpos = ChunkPosition {
                x: position.x + normal[0],
                y: position.y + normal[1],
                z: position.z + normal[2],
            };
            let faces = self.get_faces_mut(zoom_level, &neighbor_cpos, world, block_mappings);
            let i = dir as usize;
            match dir.opposite() {
                infinigen_common::world::Direction::Up => neighbor_faces[i] = faces[0],
                infinigen_common::world::Direction::Down => neighbor_faces[i] = faces[1],
                infinigen_common::world::Direction::North => neighbor_faces[i] = faces[2],
                infinigen_common::world::Direction::South => neighbor_faces[i] = faces[3],
                infinigen_common::world::Direction::East => neighbor_faces[i] = faces[4],
                infinigen_common::world::Direction::West => neighbor_faces[i] = faces[5],
            }
        }
        neighbor_faces
    }
}

pub fn get_neighbour_cposes(
    position: &ChunkPosition,
) -> impl Iterator<Item = (Direction, ChunkPosition)> + '_ {
    infinigen_common::world::Direction::iter().map(|dir| {
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
