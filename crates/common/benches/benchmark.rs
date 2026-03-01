use std::hint::black_box;

use block_mesh::ndshape::ConstShape;
use criterion::{Criterion, criterion_group, criterion_main};
use infinigen_common::mesh::block::VoxelBlock;
use infinigen_common::mesh::shapes::{PaddedChunk, PaddedChunkShape};
use infinigen_common::mesh::textures::BlockAppearances;
use infinigen_common::mesh::{mesh_chunk_greedy_quads, mesh_chunk_visible_block_faces};
use infinigen_common::world::MappedBlockID;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

fn random_padded_chunk<F>(block_constructor: F, fill_prob: f32) -> PaddedChunk
where
    F: Fn(MappedBlockID) -> VoxelBlock,
{
    let mut rng = StdRng::seed_from_u64(42);
    let mut padded = [VoxelBlock::Empty; PaddedChunkShape::USIZE];

    for voxel in padded.iter_mut() {
        let roll: f32 = rng.random();
        if roll < fill_prob {
            *voxel = block_constructor(MappedBlockID::default());
        } else {
            *voxel = VoxelBlock::Empty;
        }
    }

    padded
}

fn bench_mesh_visible_block_faces(c: &mut Criterion) {
    let padded_chunk = random_padded_chunk(VoxelBlock::Opaque, 0.3);
    let block_textures = BlockAppearances::default();

    c.bench_function("mesh_chunk_visible_block_faces", |b| {
        b.iter(|| {
            let _mesh_info = mesh_chunk_visible_block_faces(
                black_box(&padded_chunk),
                black_box(&block_textures),
            );
        });
    });
}

fn bench_mesh_greedy_quads(c: &mut Criterion) {
    let padded_chunk = random_padded_chunk(VoxelBlock::Translucent, 0.3);
    let block_textures = BlockAppearances::default();

    c.bench_function("mesh_chunk_greedy_quads", |b| {
        b.iter(|| {
            let _mesh_info =
                mesh_chunk_greedy_quads(black_box(&padded_chunk), black_box(&block_textures));
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench_mesh_visible_block_faces, bench_mesh_greedy_quads
);
criterion_main!(benches);
