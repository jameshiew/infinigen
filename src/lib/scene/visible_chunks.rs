use nalgebra::{Matrix4, Unit, Vector4};

use crate::common::chunks::CHUNK_SIZE_F32;
use crate::common::world::ChunkPosition;

/// Returns an iterator over `ChunkPosition`s within a cylinder centred at `centre` that should be loaded and rendered.
pub fn in_distance(
    centre: &ChunkPosition,
    hview_distance: usize,
    vview_distance: usize,
) -> impl Iterator<Item = ChunkPosition> {
    let mut chunks = Vec::new();
    let (hview_distance, vview_distance) = (hview_distance as i32, vview_distance as i32);
    let hview_distance_squared = hview_distance * hview_distance;
    for x in -hview_distance..=hview_distance {
        for y in -vview_distance..=vview_distance {
            for z in -hview_distance..=hview_distance {
                if x * x + z * z <= hview_distance_squared {
                    chunks.push(ChunkPosition {
                        x: centre.x + x,
                        y: centre.y + y,
                        z: centre.z + z,
                    });
                }
            }
        }
    }
    chunks.into_iter()
}

#[derive(Debug)]
pub struct Plane {
    normal: Unit<Vector4<f32>>,
    constant: f32,
}

pub fn normalize_plane(plane: Vector4<f32>) -> Plane {
    let magnitude = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
    Plane {
        normal: Unit::new_normalize(Vector4::new(plane.x, plane.y, plane.z, 0.0)),
        constant: plane.w / magnitude,
    }
}

pub fn compute_frustum_planes(combined_matrix: &Matrix4<f32>) -> [Plane; 6] {
    let m = combined_matrix;

    [
        normalize_plane(Vector4::new(
            m[(3, 0)] + m[(0, 0)],
            m[(3, 1)] + m[(0, 1)],
            m[(3, 2)] + m[(0, 2)],
            m[(3, 3)] + m[(0, 3)],
        )), // Left
        normalize_plane(Vector4::new(
            m[(3, 0)] - m[(0, 0)],
            m[(3, 1)] - m[(0, 1)],
            m[(3, 2)] - m[(0, 2)],
            m[(3, 3)] - m[(0, 3)],
        )), // Right
        normalize_plane(Vector4::new(
            m[(3, 0)] - m[(1, 0)],
            m[(3, 1)] - m[(1, 1)],
            m[(3, 2)] - m[(1, 2)],
            m[(3, 3)] - m[(1, 3)],
        )), // Top
        normalize_plane(Vector4::new(
            m[(3, 0)] + m[(1, 0)],
            m[(3, 1)] + m[(1, 1)],
            m[(3, 2)] + m[(1, 2)],
            m[(3, 3)] + m[(1, 3)],
        )), // Bottom
        normalize_plane(Vector4::new(
            m[(3, 0)] + m[(2, 0)],
            m[(3, 1)] + m[(2, 1)],
            m[(3, 2)] + m[(2, 2)],
            m[(3, 3)] + m[(2, 3)],
        )), // Near
        normalize_plane(Vector4::new(
            m[(3, 0)] - m[(2, 0)],
            m[(3, 1)] - m[(2, 1)],
            m[(3, 2)] - m[(2, 2)],
            m[(3, 3)] - m[(2, 3)],
        )), // Far
    ]
}

pub fn check_chunk_in_frustum(chunk: &ChunkPosition, frustum_planes: &[Plane; 6]) -> bool {
    for plane in frustum_planes {
        let chunk_center = Vector4::new(chunk.x as f32, chunk.y as f32, chunk.z as f32, 1.0);
        let distance = plane.normal.dot(&chunk_center) + plane.constant;
        // TODO: it would be better to check against corners of chunks
        if distance < -CHUNK_SIZE_F32 / 2. {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::scene::visible_chunks::in_distance;
    use crate::scene::*;

    #[test]
    fn test_in_distance() {
        let centre = ChunkPosition { x: 0, y: 0, z: 0 };
        let chunks: Vec<_> = in_distance(&centre, 1, 1).collect();
        assert_eq!(
            chunks,
            [
                ChunkPosition { x: -1, y: -1, z: 0 },
                ChunkPosition { x: -1, y: 0, z: 0 },
                ChunkPosition { x: -1, y: 1, z: 0 },
                ChunkPosition { x: 0, y: -1, z: -1 },
                ChunkPosition { x: 0, y: -1, z: 0 },
                ChunkPosition { x: 0, y: -1, z: 1 },
                ChunkPosition { x: 0, y: 0, z: -1 },
                ChunkPosition { x: 0, y: 0, z: 0 },
                ChunkPosition { x: 0, y: 0, z: 1 },
                ChunkPosition { x: 0, y: 1, z: -1 },
                ChunkPosition { x: 0, y: 1, z: 0 },
                ChunkPosition { x: 0, y: 1, z: 1 },
                ChunkPosition { x: 1, y: -1, z: 0 },
                ChunkPosition { x: 1, y: 0, z: 0 },
                ChunkPosition { x: 1, y: 1, z: 0 }
            ]
        );
    }
}
