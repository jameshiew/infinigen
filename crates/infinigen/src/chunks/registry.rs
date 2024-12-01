use bevy::pbr::StandardMaterial;
use rustc_hash::FxHashMap;

use bevy::prelude::{Mesh, Resource};
use strum::IntoEnumIterator;

use crate::assets::blocks::BlockMappings;
use crate::world::World;
use infinigen_common::chunks::Chunk;
use infinigen_common::mesh::faces::extract_faces;
use infinigen_common::mesh::shapes::{empty_chunk_face, ChunkFace};
use infinigen_common::world::{ChunkPosition, Direction};
use infinigen_common::zoom::ZoomLevel;

// Responsible for keeping track of chunks.
#[derive(Default, Resource)]
pub struct ChunkRegistry {
    cached: FxHashMap<ZoomLevel, FxHashMap<ChunkPosition, ChunkStatus>>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub chunk: Chunk,
    pub faces: [ChunkFace; 6],
}

impl ChunkInfo {
    pub fn empty() -> Self {
        Self {
            chunk: Chunk::Empty,
            faces: [empty_chunk_face(); 6],
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChunkStatus {
    Meshed {
        chunk_info: Box<ChunkInfo>,
        mesh: Mesh,
        material: StandardMaterial,
    },
    Generated(Box<ChunkInfo>),
    Requested,
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
                let mut chunks = FxHashMap::default();
                chunks.insert(*cpos, status);
                self.cached.insert(zoom_level, chunks);
            }
        }
    }

    pub fn fetch_and_insert(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        block_mappings: &BlockMappings,
    ) -> ChunkInfo {
        let chunk = world.get_chunk(zoom_level, position);
        self.insert_generated(zoom_level, position, chunk, block_mappings)
    }

    pub fn insert_generated(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        chunk: Chunk,
        block_mappings: &BlockMappings,
    ) -> ChunkInfo {
        let chunk_info = match chunk {
            Chunk::Empty => ChunkInfo::empty(),
            Chunk::Unpacked(chunk) => {
                let faces = extract_faces(&chunk, block_mappings);
                let chunk = (*chunk).into();
                ChunkInfo { chunk, faces }
            }
        };
        self.set_status(
            zoom_level,
            position,
            ChunkStatus::Generated(Box::new(chunk_info.clone())),
        );
        chunk_info
    }

    pub fn get_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        block_mappings: &BlockMappings,
    ) -> Chunk {
        if let Some(ChunkStatus::Generated(chunk_info)) = self.get_status(zoom_level, position) {
            let chunk = &chunk_info.chunk;
            tracing::debug!(?position, "Got cached chunk");
            return chunk.to_owned();
        }
        self.fetch_and_insert(zoom_level, position, world, block_mappings)
            .chunk
    }

    pub fn get_faces_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        block_mappings: &BlockMappings,
    ) -> [ChunkFace; 6] {
        if let Some(ChunkStatus::Generated(chunk_info)) = self.get_status(zoom_level, position) {
            let faces = chunk_info.faces;
            return faces;
        }
        self.fetch_and_insert(zoom_level, position, world, block_mappings)
            .faces
    }

    /// Returns the faces of neighboring chunks, in direction order. The bottom face of the chunk above, then the top face of the chunk below, etc.
    pub fn get_neighboring_faces_mut(
        &mut self,
        zoom_level: ZoomLevel,
        position: &ChunkPosition,
        world: &World,
        block_mappings: &BlockMappings,
    ) -> [ChunkFace; 6] {
        let mut neighbor_faces = [empty_chunk_face(); 6];
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

pub fn get_neighbour_cposes(position: &ChunkPosition) -> [(Direction, ChunkPosition); 6] {
    infinigen_common::world::Direction::iter()
        .map(|dir| {
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
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
