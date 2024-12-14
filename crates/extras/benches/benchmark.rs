use criterion::{black_box, criterion_group, criterion_main, Criterion};
use infinigen_common::blocks::BlockType;
use infinigen_common::world::{ChunkPosition, WorldGen};
use infinigen_common::zoom::ZoomLevel;
use infinigen_extras::blocks::block_types;
use infinigen_extras::worldgen::mountain_islands::MountainIslands;

fn bench_mountain_islands(c: &mut Criterion) {
    let mut wgen = MountainIslands::default();
    wgen.initialize(
        block_types()
            .enumerate()
            .map(|(chunk_block_id, BlockType { id, .. })| (id, chunk_block_id as u8))
            .collect(),
    );
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
