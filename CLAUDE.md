# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Infinigen is a Minecraft-like procedural terrain generation project built with Bevy game engine in Rust. It focuses on efficient chunk rendering and procedural world generation without gameplay elements.

## Development Commands

Use `just` for all development tasks:

### Essential Commands

- `just check` - Check compilation (use this often during development)
- `just test` - Run tests with cargo nextest
- `just clippy` - Run linting (must pass before committing)
- `just fmt` - Format code
- `just lint` - Run both format check and clippy
- `just release` - Build and run in release mode
- `just debug` - Build and run in debug mode

### Before Committing

Always run in this order:

1. `just clippy` - Fix all errors
2. `just fmt` - Format code

### Other Useful Commands

- `just doc` - Build documentation
- `just run-wasm` - Run WebAssembly build
- `just screenshot-and-exit` - Test with screenshot
- `just machete` - Check for unused dependencies
- `just audit` - Security audit dependencies

## Architecture

### Crate Structure

1. **`crates/common/`** - Core logic (no Bevy dependencies)

   - Block definitions and chunk management
   - Mesh generation algorithms
   - World generation traits

2. **`crates/plugins/`** - Bevy plugin implementations

   - `AppPlugin` orchestrates all sub-plugins
   - Key plugins: Registry, Assets, Scene, Mesh, Camera, World, Debug

3. **`crates/extras/`** - Default implementations

   - Concrete block types and textures
   - World generators (MountainIslands, Flat, SingleBlock)

4. **`crates/infinigen/`** - Main executable entry point

### Key Architectural Patterns

**State Flow:**

```
LoadingAssets → InitializingRegistry → InitializingWorld → MainGame
```

**Chunk Generation Pipeline:**

```
Camera moves → Scene update → Generate chunk request →
World generator creates data → Mesh generation → Spawn entities
```

**Message System:**

- `GenerateChunkRequest/Task` - World generation messages
- `MeshChunkRequest/Rerequest` - Mesh generation messages
- `UpdateSceneMessage` - Scene update triggers

**Resource Management:**

- `BlockRegistry` - Central block type registry
- `World` - World generator and chunk cache
- `Meshes` - Generated mesh cache

## Rust Development Guidelines

### Code Style

- Keep modules small
- Use `format!("{var}")` over `format!("{}", var)`
- Guard against numeric overflow with saturating operations
- Compile often with `just check`

### Adding Features

**New Block Type:**

1. Add RON file in `crates/extras/assets/blocks/types/`
2. Add texture in `crates/extras/assets/blocks/textures/`
3. Update block registry

**New World Generator:**

1. Implement `WorldGen` trait in `crates/extras/src/worldgen/`
2. Add to `WorldInitializer` in `crates/extras/src/world_initializer.rs`
3. Update config options

**New Plugin:**

1. Create in `crates/plugins/src/`
2. Add to `AppPlugin` in `crates/plugins/src/app.rs`
3. Follow existing plugin patterns for state and messages

## Testing Approach

- Unit tests: Located alongside code files
- Integration tests: Use `just test`
- Visual testing: Use `just screenshot-and-exit`
- WASM testing: Use `just run-wasm`

## Performance Considerations

- Chunks use u8 for block IDs (max 256 block types)
- Parallel chunk/mesh generation via Bevy tasks
- Zoom levels for distant chunk optimization
- Separate opaque/translucent rendering passes

## Configuration

Config via YAML or environment variables:

- `hview_distance` - Horizontal view distance in chunks
- `vview_distance` - Vertical view distance in chunks
- `world` - World type (MountainIslands, Flat, SingleBlock)
- `zoom_level` - Level of detail
- `seed` - World generation seed

Environment variable: `INFINIGEN_WORLD=Flat cargo run --release`
