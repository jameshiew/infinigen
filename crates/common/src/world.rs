use std::fmt;
use std::num::NonZeroU8;

use linearize::Linearize;
use strum::EnumIter;

use crate::chunks::{Chunk, CHUNK_SIZE, CHUNK_SIZE_F32, CHUNK_SIZE_I32};
use crate::zoom::ZoomLevel;

/// Chunks work with [`MappedBlockID`]s (u8s), which correspond to [`crate::blocks::BlockID`]s (strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappedBlockID(NonZeroU8);

impl Default for MappedBlockID {
    fn default() -> Self {
        Self(unsafe { NonZeroU8::new_unchecked(1) })
    }
}

impl TryFrom<u8> for MappedBlockID {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        NonZeroU8::new(value)
            .map(Self)
            .ok_or("mapped block ID cannot be zero")
    }
}

impl MappedBlockID {
    pub fn next(&self) -> Option<Self> {
        if self.0 == NonZeroU8::MAX {
            None
        } else {
            Some(Self(self.0.checked_add(1).unwrap()))
        }
    }
}

pub trait WorldGen {
    fn get(&self, pos: &ChunkPosition, zoom_level: ZoomLevel) -> Chunk;
}

/// Position of a block within a chunk.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BlockPosition {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl BlockPosition {
    pub const MAX_IDX: u8 = CHUNK_SIZE - 1;
    pub const MIN_IDX: u8 = 0;
}

/// Position of a chunk within a world.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl fmt::Display for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Absolute position of something within the world (e.g. a camera)
#[derive(Debug, Clone, Copy)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Absolute position of a block in the world.
#[derive(Debug, Clone, Copy)]
pub struct WorldBlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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

impl From<&ChunkPosition> for WorldBlockPosition {
    fn from(cpos: &ChunkPosition) -> Self {
        Self {
            x: cpos.x * CHUNK_SIZE_I32,
            y: cpos.y * CHUNK_SIZE_I32,
            z: cpos.z * CHUNK_SIZE_I32,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter, Linearize)]
#[linearize(const)]
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
