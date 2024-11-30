use criterion::{black_box, criterion_group, criterion_main, Criterion};
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;

use infinigen_common::extras::block_ids::default_block_ids;
use infinigen_common::extras::worldgen::layered::Layered;

fn bench_layered(c: &mut Criterion) {
    let mut wgen = Layered::default();
    wgen.initialize(default_block_ids());
    const SIZE: i32 = 3;
    c.bench_function("layered", |b| {
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

criterion_group!(benches, bench_layered,);
criterion_main!(benches);
