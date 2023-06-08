# infinigen

This is a demo for Minecraft-like procedural generation using the [Bevy game engine](https://bevyengine.org/).

- chunks along all axes (X, Y and Z)
- adjustable zoom level for viewing a world at different levels of detail

![Main screenshot](screenshots/main.png "Screenshot")
![Zoomed out screenshot](screenshots/zoomed_out.png "Zoomed out")

## Quickstart

```shell
cargo run --release --no-default-features
```

or

```shell
cargo run  # dynamically linked, compiles faster and runs slower
```

### Controls

- <kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> - to move around
- <kbd>Space</kbd> - ascend
- <kbd>Shift</kbd> - descend
- <kbd>F3</kbd> - toggle wireframes
- <kbd>F9</kbd> - toggle chunk borders (only works near the origin)

### Configuration

Copy `config.example.ron` to `config.ron` if you want to adjust settings beforehand (e.g. initial start position).

## Development

A nightly Rust version is used only to make compilation faster and to be able to run benchmarks with `cargo bench`. It should remain possible though to build release binaries using Rust stable.

All textures are derived from images generated with [Midjourney](https://midjourney.com).
