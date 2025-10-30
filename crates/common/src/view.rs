use std::cmp::Ordering;

use ahash::AHashSet;
use nalgebra::{Matrix4, Perspective3, Quaternion, Unit, UnitQuaternion, Vector3, Vector4};

use crate::chunks::CHUNK_SIZE_F32;
use crate::world::ChunkPosition;

/// Calculates the squared distance between two chunk positions.
#[inline]
const fn chunk_distance_squared(from: &ChunkPosition, to: &ChunkPosition) -> i32 {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    dx * dx + dy * dy + dz * dz
}

/// Returns an iterator over `ChunkPosition`s within a cylinder centred at `centre` that should be loaded and rendered.
pub fn in_distance(
    centre: &ChunkPosition,
    horizontal_view_distance: usize,
    vertical_view_distance: usize,
) -> impl Iterator<Item = ChunkPosition> {
    let mut chunks = Vec::new();
    let (horizontal_view_distance, vertical_view_distance) = (
        horizontal_view_distance as i32,
        vertical_view_distance as i32,
    );
    let horizontal_view_distance_squared = horizontal_view_distance * horizontal_view_distance;
    for x in -horizontal_view_distance..=horizontal_view_distance {
        for y in -vertical_view_distance..=vertical_view_distance {
            for z in -horizontal_view_distance..=horizontal_view_distance {
                if x * x + z * z <= horizontal_view_distance_squared {
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
    let magnitude = plane
        .z
        .mul_add(plane.z, plane.x.mul_add(plane.x, plane.y * plane.y))
        .sqrt();
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

// TODO: too many arguments
pub fn compute_chunks_delta(
    current_cpos: ChunkPosition,
    horizontal_view_distance: usize,
    vertical_view_distance: usize,
    camera_translation: [f32; 3],
    camera_rotation: [f32; 4], // w,x,y,z
    aspect_ratio: f32,
    fov: f32,
    near: f32,
    far: f32,
    already_loaded_or_loading: &AHashSet<ChunkPosition>,
) -> (Vec<ChunkPosition>, Vec<ChunkPosition>) {
    let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
        camera_rotation[0],
        camera_rotation[1],
        camera_rotation[2],
        camera_rotation[3],
    ));
    let translation = Vector3::new(
        camera_translation[0],
        camera_translation[1],
        camera_translation[2],
    );

    let persp_proj = Perspective3::new(aspect_ratio, fov, near, far);
    let projection_matrix: Matrix4<f32> = *persp_proj.as_matrix();
    let view_matrix: Matrix4<f32> =
        Matrix4::from(rotation.to_rotation_matrix()).append_translation(&-translation);
    let combined_matrix = projection_matrix * view_matrix;
    let frustum_planes = compute_frustum_planes(&combined_matrix);

    let chunks_within_render_distance: AHashSet<_> = in_distance(
        &current_cpos,
        horizontal_view_distance,
        vertical_view_distance,
    )
    .collect();

    let mut to_load: Vec<_> = chunks_within_render_distance
        .difference(already_loaded_or_loading)
        .copied()
        .collect();

    // nearest chunks first
    to_load.sort_unstable_by_key(|c| chunk_distance_squared(&current_cpos, c));

    // chunks within view frustum first
    to_load.sort_unstable_by(|c1, c2| {
        let in_frustum1 = check_chunk_in_frustum(c1, &frustum_planes);
        let in_frustum2 = check_chunk_in_frustum(c2, &frustum_planes);

        if in_frustum1 && !in_frustum2 {
            Ordering::Less
        } else if !in_frustum1 && in_frustum2 {
            Ordering::Greater
        } else {
            let dist1 = chunk_distance_squared(&current_cpos, c1);
            let dist2 = chunk_distance_squared(&current_cpos, c2);
            dist1.cmp(&dist2)
        }
    });

    let to_unload: Vec<_> = already_loaded_or_loading
        .difference(&chunks_within_render_distance)
        .copied()
        .collect();

    (to_load, to_unload)
}

#[cfg(test)]
mod tests {
    use super::*;

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
