use criterion::{black_box, criterion_group, criterion_main, Criterion};
use infinigen::common::world::{ChunkPosition, WorldGen};
use infinigen::common::zoom::ZoomLevel;

use infinigen::extras::worldgen::flat::Flat;
use infinigen::extras::worldgen::mountain_archipelago::MountainIslands;
use infinigen::extras::worldgen::perlin_noise::PerlinNoise;
use infinigen::extras::{block_ids::default_block_ids, chunks::filled_chunk};

fn bench_filled_chunk(c: &mut Criterion) {
    c.bench_function("filled chunk", |b| b.iter(|| filled_chunk(black_box(0))));
}

fn bench_flat(c: &mut Criterion) {
    let mut wgen = Flat::default();
    wgen.initialize(default_block_ids());
    let underground = ChunkPosition { x: 0, y: -1, z: 0 };
    c.bench_function("flat", |b| {
        b.iter(|| wgen.get(black_box(&underground), black_box(ZoomLevel::default())))
    });
}

fn bench_perlin_noise(c: &mut Criterion) {
    let mut wgen = PerlinNoise::default();
    wgen.initialize(default_block_ids());
    c.bench_function("perlin noise", |b| {
        b.iter(|| {
            wgen.get(
                black_box(&ChunkPosition::default()),
                black_box(ZoomLevel::default()),
            )
        })
    });
}

fn bench_mountain_archipelago(c: &mut Criterion) {
    // TODO: vary seed and chunk position?
    let mut wgen = MountainIslands::default();
    wgen.initialize(default_block_ids());
    c.bench_function("mountain archipelago", |b| {
        b.iter(|| {
            wgen.get(
                black_box(&ChunkPosition::default()),
                black_box(ZoomLevel::default()),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_filled_chunk,
    bench_flat,
    bench_perlin_noise,
    bench_mountain_archipelago
);
criterion_main!(benches);
