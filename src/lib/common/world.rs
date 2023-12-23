use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::common::chunks::{Chunk, CHUNK_SIZE, CHUNK_SIZE_F32, CHUNK_SIZE_I32};
use crate::common::zoom::ZoomLevel;

pub type BlockId = String;
pub type ChunkBlockId = u8;

pub trait WorldGen {
    /// Must be called before getting any chunks. If a world gen depends on a [`BlockId`] for which there is no [`ChunkBlockId`] provided, it may panic!
    fn initialize(&mut self, mappings: HashMap<BlockId, ChunkBlockId>);
    fn get(&mut self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum BlockVisibility {
    #[default]
    Opaque,
    Translucent,
}

/// Position of a block within a chunk. We use i8 for coordinates to make arithmetic easier when meshing, which may not be the best reason.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BlockPosition {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl BlockPosition {
    pub const MAX_IDX: i8 = CHUNK_SIZE - 1;
    pub const MIN_IDX: i8 = 0;
}

/// Position of a chunk within a world.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Absolute position of something within the world (e.g. a camera)
#[derive(Debug, Clone, Copy)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Position of something relative to its own space. i.e. not the entire world
#[derive(Debug, Clone, Copy)]
pub struct LocalPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<&ChunkPosition> for WorldPosition {
    fn from(cpos: &ChunkPosition) -> Self {
        Self {
            x: (cpos.x * CHUNK_SIZE_I32) as f32,
            y: (cpos.y * CHUNK_SIZE_I32) as f32,
            z: (cpos.z * CHUNK_SIZE_I32) as f32,
        }
    }
}

impl From<WorldPosition> for ChunkPosition {
    fn from(wpos: WorldPosition) -> Self {
        Self {
            x: (wpos.x / CHUNK_SIZE_F32).floor() as i32,
            y: (wpos.y / CHUNK_SIZE_F32).floor() as i32,
            z: (wpos.z / CHUNK_SIZE_F32).floor() as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter)]
pub enum Direction {
    Up = 0,
    Down,
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

impl From<Direction> for [i32; 3] {
    /// The normalized vector for this direction, in a right-handed Y-up coordinate system.
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Up => [0, 1, 0],
            Direction::Down => [0, -1, 0],
            Direction::North => [0, 0, -1],
            Direction::South => [0, 0, 1],
            Direction::East => [1, 0, 0],
            Direction::West => [-1, 0, 0],
        }
    }
}
