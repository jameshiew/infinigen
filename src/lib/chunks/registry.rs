use std::collections::HashMap;

use bevy::prelude::Resource;
use strum::IntoEnumIterator;

use crate::common::chunks::Chunk;
use crate::common::world::ChunkPosition;
use crate::fake_client::FakeClient;
use crate::mesh::faces::extract_faces;
use crate::mesh::shapes::{empty_chunk_face, ChunkFace};
use crate::scene::assets::BlockMappings;

type ZoomLevel = i8;

// Responsible for keeping track of chunks which have been received by this local client.
#[derive(Default, Resource)]
pub struct ChunkRegistry {
    cached: HashMap<ZoomLevel, HashMap<ChunkPosition, ChunkStatus>>,
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
    Present(Box<ChunkInfo>),
    Generating,
}

impl ChunkRegistry {
    pub fn clear(&mut self) {
        self.cached.clear();
    }

    pub fn get_status(&self, zoom_level: &ZoomLevel, cpos: &ChunkPosition) -> Option<ChunkStatus> {
        match self.cached.get(zoom_level) {
            Some(chunks) => chunks.get(cpos).map(|status| status.to_owned()),
            None => None,
        }
    }

    pub fn set_status(
        &mut self,
        zoom_level: &ZoomLevel,
        cpos: &ChunkPosition,
        status: ChunkStatus,
    ) {
        match self.cached.get_mut(zoom_level) {
            Some(chunks) => {
                chunks.insert(*cpos, status);
            }
            None => {
                let mut chunks = HashMap::new();
                chunks.insert(*cpos, status);
                self.cached.insert(*zoom_level, chunks);
            }
        }
    }

    pub fn fetch_and_insert(
        &mut self,
        zoom_level: &ZoomLevel,
        position: &ChunkPosition,
        client: &FakeClient,
        block_mappings: &BlockMappings,
    ) -> ChunkInfo {
        let chunk = client.get_chunk(*zoom_level, position);
        self.insert(zoom_level, position, chunk, block_mappings)
    }

    pub fn insert(
        &mut self,
        zoom_level: &ZoomLevel,
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
            ChunkStatus::Present(Box::new(chunk_info.clone())),
        );
        chunk_info
    }

    pub fn get_mut(
        &mut self,
        zoom_level: &ZoomLevel,
        position: &ChunkPosition,
        client: &FakeClient,
        block_mappings: &BlockMappings,
    ) -> Chunk {
        if let Some(ChunkStatus::Present(chunk_info)) = self.get_status(zoom_level, position) {
            let chunk = &chunk_info.chunk;
            tracing::debug!(?position, "Got cached chunk");
            return chunk.to_owned();
        }
        self.fetch_and_insert(zoom_level, position, client, block_mappings)
            .chunk
    }

    pub fn get_faces_mut(
        &mut self,
        zoom_level: &ZoomLevel,
        position: &ChunkPosition,
        client: &FakeClient,
        block_mappings: &BlockMappings,
    ) -> [ChunkFace; 6] {
        if let Some(ChunkStatus::Present(chunk_info)) = self.get_status(zoom_level, position) {
            let faces = chunk_info.faces;
            return faces;
        }
        self.fetch_and_insert(zoom_level, position, client, block_mappings)
            .faces
    }

    /// Returns the faces of neighboring chunks, in direction order. The bottom face of the chunk above, then the top face of the chunk below, etc.
    pub fn get_neighboring_faces_mut(
        &mut self,
        zoom_level: &ZoomLevel,
        position: &ChunkPosition,
        client: &FakeClient,
        block_mappings: &BlockMappings,
    ) -> [ChunkFace; 6] {
        let mut neighbor_faces = [empty_chunk_face(); 6];
        for dir in crate::common::world::Direction::iter() {
            let normal: [i32; 3] = dir.into();
            let neighbor_cpos = ChunkPosition {
                x: position.x + normal[0],
                y: position.y + normal[1],
                z: position.z + normal[2],
            };
            let faces = self.get_faces_mut(zoom_level, &neighbor_cpos, client, block_mappings);
            let i = dir as usize;
            match dir.opposite() {
                crate::common::world::Direction::Up => neighbor_faces[i] = faces[0],
                crate::common::world::Direction::Down => neighbor_faces[i] = faces[1],
                crate::common::world::Direction::North => neighbor_faces[i] = faces[2],
                crate::common::world::Direction::South => neighbor_faces[i] = faces[3],
                crate::common::world::Direction::East => neighbor_faces[i] = faces[4],
                crate::common::world::Direction::West => neighbor_faces[i] = faces[5],
            }
        }
        neighbor_faces
    }
}
