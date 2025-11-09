use std::cmp::Ordering;

use ahash::AHashSet;
use glam::{Mat4, Quat, Vec3, Vec4};

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
    normal: Vec4,
    constant: f32,
}

pub fn normalize_plane(plane: Vec4) -> Plane {
    let normal = plane.truncate();
    let magnitude = normal.length();

    if magnitude == 0.0 {
        // Degenerate plane; fall back to zero normal to avoid NaNs.
        return Plane {
            normal: Vec4::ZERO,
            constant: 0.0,
        };
    }

    let normalized = Vec4::new(
        normal.x / magnitude,
        normal.y / magnitude,
        normal.z / magnitude,
        0.0,
    );
    Plane {
        normal: normalized,
        constant: plane.w / magnitude,
    }
}

pub fn compute_frustum_planes(combined_matrix: &Mat4) -> [Plane; 6] {
    let cols = combined_matrix.to_cols_array();
    let row = |r: usize| -> Vec4 { Vec4::new(cols[r], cols[4 + r], cols[8 + r], cols[12 + r]) };

    let row0 = row(0);
    let row1 = row(1);
    let row2 = row(2);
    let row3 = row(3);

    [
        normalize_plane(row3 + row0), // Left
        normalize_plane(row3 - row0), // Right
        normalize_plane(row3 - row1), // Top
        normalize_plane(row3 + row1), // Bottom
        normalize_plane(row3 + row2), // Near
        normalize_plane(row3 - row2), // Far
    ]
}

pub fn check_chunk_in_frustum(chunk: &ChunkPosition, frustum_planes: &[Plane; 6]) -> bool {
    for plane in frustum_planes {
        let chunk_center = Vec4::new(chunk.x as f32, chunk.y as f32, chunk.z as f32, 1.0);
        let distance = plane.normal.dot(chunk_center) + plane.constant;
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
    let rotation = Quat::from_xyzw(
        camera_rotation[1],
        camera_rotation[2],
        camera_rotation[3],
        camera_rotation[0],
    )
    .normalize();
    let translation = Vec3::new(
        camera_translation[0],
        camera_translation[1],
        camera_translation[2],
    );

    let projection_matrix = Mat4::perspective_rh_gl(fov, aspect_ratio, near, far);
    let view_matrix = Mat4::from_quat(rotation) * Mat4::from_translation(-translation);
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
