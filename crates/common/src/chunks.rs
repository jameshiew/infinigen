use std::fmt;

use ndarray::Array3;
use serde::{Deserialize, Serialize};

use crate::world::BlockPosition;

use super::world::ChunkBlockId;

/// The length of one side of a cubic chunk.
pub const CHUNK_SIZE: i8 = 32;

pub const CHUNK_USIZE: usize = CHUNK_SIZE as usize;
pub const CHUNK_SIZE_U32: u32 = CHUNK_SIZE as u32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const CHUNK_SIZE_F64: f64 = CHUNK_SIZE as f64;

#[derive(Default, Clone)]
pub enum Chunk {
    #[default]
    Empty,
    Unpacked(Box<UnpackedChunk>),
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chunk::Empty => write!(f, "Chunk::Empty"),
            Chunk::Unpacked(chunk) => write!(f, "Chunk::Unpacked({})", chunk.count_not_empty()),
        }
    }
}

impl From<UnpackedChunk> for Chunk {
    fn from(chunk: UnpackedChunk) -> Self {
        // we don't check if the chunk is empty here because that's expensive?
        Chunk::Unpacked(Box::new(chunk))
    }
}

impl From<Chunk> for UnpackedChunk {
    fn from(value: Chunk) -> Self {
        match value {
            Chunk::Unpacked(chunk) => *chunk,
            Chunk::Empty => UnpackedChunk::default(),
        }
    }
}

/// A chunk of the world. `None` blocks represent empty blocks.
/// Explicitly not `Copy` as copying could be expensive.
/// We don't derive `Debug` or `Default`, so that we can use chunk sizes greater than 32.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnpackedChunk {
    blocks: Array3<Option<ChunkBlockId>>,
}

impl Default for UnpackedChunk {
    fn default() -> Self {
        Self {
            blocks: Array3::<Option<ChunkBlockId>>::from_elem(
                (CHUNK_USIZE, CHUNK_USIZE, CHUNK_USIZE),
                None,
            ),
        }
    }
}

impl UnpackedChunk {
    pub fn get(&self, pos: &BlockPosition) -> Option<ChunkBlockId> {
        self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]]
    }

    pub fn insert(&mut self, pos: &BlockPosition, block: ChunkBlockId) {
        self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]] = Some(block);
    }

    pub fn insert_if_free(&mut self, pos: &BlockPosition, block: ChunkBlockId) {
        if self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]].is_none() {
            self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]] = Some(block);
        }
    }

    pub fn clear(&mut self, pos: &BlockPosition) {
        self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]] = None;
    }

    pub fn count_not_empty(&self) -> usize {
        let mut count = 0;
        for x in 0..CHUNK_USIZE {
            for y in 0..CHUNK_USIZE {
                for z in 0..CHUNK_USIZE {
                    if self.blocks[[x, y, z]].is_some() {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn is_empty(&self) -> bool {
        self.count_not_empty() == 0
    }
}
