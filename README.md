# infinigen [![CI](https://github.com/jameshiew/infinigen/actions/workflows/ci.yml/badge.svg)](https://github.com/jameshiew/infinigen/actions/workflows/ci.yml)

This is a demo for Minecraft-like procedural generation using the [Bevy game engine](https://bevyengine.org/).

- chunks along all axes (X, Y and Z)
- adjustable zoom level for viewing a world at different levels of detail

![Main screenshot](screenshots/main.webp "Screenshot")
![Zoomed out screenshot](screenshots/zoomed_out.webp "Zoomed out")

## Quickstart

These commands must be run from within the root directory of the repo, in order to pick up assets.

```shell
cargo run --release # best performance
cargo run --release --features jemalloc  # on macOS/Linux
```

or

```shell
cargo run --features bevy/dynamic_linking  # compiles fastest
```

### Controls

- <kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> - to move around
- <kbd>Space</kbd> - ascend
- <kbd>Shift</kbd> - descend
- <kbd>F3</kbd> - toggle wireframes
- <kbd>F7</kbd> - toggle debug panels

### Configuration

Copy `config.example.ron` to `config.ron` if you want to adjust settings beforehand (e.g. initial start position).

## Development

All textures are derived from images generated with [Midjourney](https://midjourney.com).
