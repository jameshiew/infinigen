use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use eyre::Result;

use crate::common::chunks::{Chunk, UnpackedChunk};
use crate::common::world::{ChunkPosition, WorldGen};
use crate::common::zoom::ZoomLevel;

/// Read/write chunks. A world is stored in a folder with chunks named like `x.y.z.chunk`. The chunks are simply stored using bincode.
pub struct Backend {
    path: PathBuf,
}

impl Backend {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn chunk_path(&self, pos: &ChunkPosition) -> PathBuf {
        self.path
            .join(format!("{}.{}.{}.chunk", pos.x, pos.y, pos.z))
    }

    pub fn chunk_exists(&self, pos: &ChunkPosition) -> Result<bool> {
        let chunk_path = self.chunk_path(pos);
        let exists = chunk_path.try_exists()?;
        Ok(exists)
    }

    pub fn read(&self, pos: &ChunkPosition) -> Result<UnpackedChunk> {
        let chunk_path = self.chunk_path(pos);
        let file = std::fs::File::open(chunk_path)?;
        if file.metadata()?.len() == 0 {
            return Ok(UnpackedChunk::default());
        }
        let chunk = bincode::deserialize_from(file)?;
        Ok(chunk)
    }

    pub fn write(&self, pos: &ChunkPosition, chunk: &UnpackedChunk) -> Result<()> {
        let chunk_path = self.chunk_path(pos);
        if let Some(parent) = chunk_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(chunk_path)?;
        if chunk.is_empty() {
            return Ok(()); // we use an empty file to represent an empty chunk
        }
        bincode::serialize_into(file, chunk)?;
        Ok(())
    }
}

/// Stores chunks using the given backend, but also generates and persists them on the fly if they aren't present.
pub struct PersistentWorld {
    backend: Backend,
    generator: Arc<RwLock<Box<dyn WorldGen + Send + Sync>>>,
}

impl PersistentWorld {
    pub fn new(backend: Backend, generator: Arc<RwLock<Box<dyn WorldGen + Send + Sync>>>) -> Self {
        Self { backend, generator }
    }
}

impl WorldGen for PersistentWorld {
    fn initialize(
        &mut self,
        _mappings: rustc_hash::FxHashMap<
            crate::common::world::BlockId,
            crate::common::world::ChunkBlockId,
        >,
    ) {
        todo!()
    }

    /// Attempts to get the chunk from disk. Does not propagate errors, only logs warnings and returns None.
    fn get(&mut self, pos: &ChunkPosition, zoom: ZoomLevel) -> Chunk {
        let exists = match self.backend.chunk_exists(pos) {
            Ok(exists) => exists,
            Err(err) => {
                tracing::warn!(?err, ?pos, "Failed to check if chunk exists");
                return Chunk::Empty;
            }
        };
        if exists {
            return match self.backend.read(pos) {
                Ok(chunk) => chunk.into(),
                Err(err) => {
                    tracing::warn!(?err, ?pos, "Failed to read chunk from disk");
                    Chunk::Empty
                }
            };
        }

        // no chunk found, so attempt to generate it and persist it
        tracing::info!(?pos, "Generating and persisting chunk");
        let chunk = self.generator.write().unwrap().get(pos, zoom);
        match chunk {
            Chunk::Unpacked(chunk) => match self.backend.write(pos, &chunk) {
                Ok(()) => {
                    tracing::info!(?pos, "Finished generating and persisting chunk");
                    Chunk::Unpacked(chunk)
                }
                Err(err) => {
                    tracing::warn!(?err, ?pos, "Failed to write chunk to disk");
                    Chunk::Empty
                }
            },
            // TODO: self.backend.write should accept Chunk instead of RawChunk so that it doesn't have to check if things are empty
            Chunk::Empty => match self.backend.write(pos, &UnpackedChunk::default()) {
                Ok(()) => Chunk::Empty,
                Err(err) => {
                    tracing::warn!(?err, ?pos, "Failed to write empty chunk to disk");
                    Chunk::Empty
                }
            },
        }
    }
}
