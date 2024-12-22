use std::fmt;

use ndarray::Array3;

use super::world::MappedBlockID;
use crate::world::BlockPosition;

/// The length of one side of a cubic chunk.
pub const CHUNK_SIZE: u8 = 32;

pub const CHUNK_USIZE: usize = CHUNK_SIZE as usize;
pub const CHUNK_SIZE_U32: u32 = CHUNK_SIZE as u32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const CHUNK_SIZE_F64: f64 = CHUNK_SIZE as f64;

#[derive(Default, Clone)]
pub enum Chunk {
    #[default]
    Empty,
    Array3(Box<Array3Chunk>),
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chunk::Empty => write!(f, "Chunk::Empty"),
            Chunk::Array3(_) => write!(f, "Chunk::Array3"),
        }
    }
}

impl From<Array3Chunk> for Chunk {
    fn from(chunk: Array3Chunk) -> Self {
        Chunk::Array3(Box::new(chunk))
    }
}

impl From<Chunk> for Array3Chunk {
    fn from(value: Chunk) -> Self {
        match value {
            Chunk::Array3(chunk) => *chunk,
            Chunk::Empty => Array3Chunk::default(),
        }
    }
}

/// Chunk represented as a 3D array of [`MappedBlockID`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Array3Chunk {
    blocks: Array3<Option<MappedBlockID>>,
}

impl fmt::Debug for Array3Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Array3Chunk")
    }
}

impl Default for Array3Chunk {
    fn default() -> Self {
        Self {
            blocks: Array3::<Option<MappedBlockID>>::from_elem(
                (CHUNK_USIZE, CHUNK_USIZE, CHUNK_USIZE),
                None,
            ),
        }
    }
}

impl Array3Chunk {
    pub fn get(&self, pos: &BlockPosition) -> Option<MappedBlockID> {
        self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]]
    }

    pub fn insert(&mut self, pos: &BlockPosition, block: MappedBlockID) {
        self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]] = Some(block);
    }

    pub fn insert_if_free(&mut self, pos: &BlockPosition, block: MappedBlockID) {
        if self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]].is_none() {
            self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]] = Some(block);
        }
    }

    pub fn clear(&mut self, pos: &BlockPosition) {
        self.blocks[[pos.x as usize, pos.y as usize, pos.z as usize]] = None;
    }
}

pub fn filled_chunk(block: MappedBlockID) -> Array3Chunk {
    let mut chunk = Array3Chunk::default();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.insert(&BlockPosition { x, y, z }, block);
            }
        }
    }
    chunk
}

/// Chunk where the topmost layer is dirt - can be used to represent the ground.
pub fn top_chunk(block: MappedBlockID) -> Array3Chunk {
    let mut chunk = Array3Chunk::default();
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            chunk.insert(
                &BlockPosition {
                    x,
                    y: CHUNK_SIZE - 1,
                    z,
                },
                block,
            );
        }
    }
    chunk
}
