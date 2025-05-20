use std::hint::black_box;

use ahash::AHashMap;
use criterion::{Criterion, criterion_group, criterion_main};
use infinigen_common::blocks::{BlockType, Palette};
use infinigen_common::world::{ChunkPosition, MappedBlockID, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use infinigen_extras::blocks::block_types;
use infinigen_extras::worldgen::mountain_islands::MountainIslands;

fn bench_mountain_islands(c: &mut Criterion) {
    let mapping: AHashMap<_, _> = block_types()
        .enumerate()
        .map(|(chunk_block_id, BlockType { id, .. })| {
            (
                id,
                MappedBlockID::try_from(1 + chunk_block_id as u8).unwrap(),
            )
        })
        .collect();
    let palette: Palette = mapping.into();
    let wgen = MountainIslands::new(0, palette);
    const SIZE: i32 = 4;
    c.bench_function("mountain islands", |b| {
        b.iter(|| {
            for cx in -SIZE..SIZE {
                for cy in -SIZE..SIZE {
                    for cz in -SIZE..SIZE {
                        wgen.get(
                            black_box(&ChunkPosition {
                                x: cx,
                                y: cy,
                                z: cz,
                            }),
                            black_box(ZoomLevel::default()),
                        );
                    }
                }
            }
        })
    });
}

criterion_group!(benches, bench_mountain_islands,);
criterion_main!(benches);
