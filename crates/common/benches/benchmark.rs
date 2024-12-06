use criterion::{black_box, criterion_group, criterion_main, Criterion};
use infinigen_common::extras::block_ids::default_block_ids;
use infinigen_common::extras::worldgen::mountain_islands::MountainIslands;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;

fn bench_mountain_islands(c: &mut Criterion) {
    let mut wgen = MountainIslands::default();
    wgen.initialize(default_block_ids());
    const SIZE: i32 = 3;
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
