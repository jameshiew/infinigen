//! Axis-aligned face directions, quads, and per-face geometry helpers used
//! by the chunk meshing algorithms in [`super`].

use crate::blocks::Face;

/// One of the six axis-aligned face directions on a cube.
///
/// Discriminants match the index used in the `[Vec<Quad>; 6]` buffer returned
/// by the meshing algorithms.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceDir {
    XNeg = 0,
    XPos = 1,
    YNeg = 2,
    YPos = 3,
    ZNeg = 4,
    ZPos = 5,
}

impl FaceDir {
    pub const ALL: [Self; 6] = [
        Self::XNeg,
        Self::XPos,
        Self::YNeg,
        Self::YPos,
        Self::ZNeg,
        Self::ZPos,
    ];

    #[inline]
    pub const fn normal(self) -> [i32; 3] {
        match self {
            Self::XNeg => [-1, 0, 0],
            Self::XPos => [1, 0, 0],
            Self::YNeg => [0, -1, 0],
            Self::YPos => [0, 1, 0],
            Self::ZNeg => [0, 0, -1],
            Self::ZPos => [0, 0, 1],
        }
    }

    #[inline]
    pub const fn normal_f32(self) -> [f32; 3] {
        let [x, y, z] = self.normal();
        [x as f32, y as f32, z as f32]
    }

    /// Axis index (0=X, 1=Y, 2=Z) along which this face's normal points.
    #[inline]
    pub const fn normal_axis(self) -> usize {
        match self {
            Self::XNeg | Self::XPos => 0,
            Self::YNeg | Self::YPos => 1,
            Self::ZNeg | Self::ZPos => 2,
        }
    }

    /// Axis index along which quad width extends.
    ///
    /// Axes follow the convention used by
    /// [`block-mesh-rs`](https://github.com/bonsairobo/block-mesh-rs)'s
    /// `RIGHT_HANDED_Y_UP_CONFIG`, so that texture orientation matches what
    /// the existing texture atlas expects.
    #[inline]
    pub const fn u_axis(self) -> usize {
        match self {
            Self::XNeg | Self::XPos => 2, // Z
            Self::YNeg | Self::YPos => 2, // Z
            Self::ZNeg | Self::ZPos => 0, // X
        }
    }

    /// Axis index along which quad height extends.
    #[inline]
    pub const fn v_axis(self) -> usize {
        match self {
            Self::XNeg | Self::XPos => 1, // Y
            Self::YNeg | Self::YPos => 0, // X
            Self::ZNeg | Self::ZPos => 1, // Y
        }
    }

    #[inline]
    pub const fn is_positive(self) -> bool {
        matches!(self, Self::XPos | Self::YPos | Self::ZPos)
    }

    /// The corresponding [`Face`] used for texture atlas lookup.
    #[inline]
    pub const fn block_face(self) -> Face {
        match self {
            Self::XPos => Face::Right,
            Self::XNeg => Face::Left,
            Self::YPos => Face::Top,
            Self::YNeg => Face::Bottom,
            Self::ZPos => Face::Back,
            Self::ZNeg => Face::Front,
        }
    }
}

/// A face quad covering `width × height` voxels on one face of the voxel at
/// [`voxel`](Quad::voxel).
///
/// Width extends along [`FaceDir::u_axis`]; height extends along
/// [`FaceDir::v_axis`]. Unit quads have `width == height == 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    /// Minimum-corner voxel in the quad, in padded chunk coordinates.
    pub voxel: [u32; 3],
    pub width: u32,
    pub height: u32,
}

impl Quad {
    /// Returns the 4 vertex positions of the quad, in the order:
    /// `[min_u/min_v, max_u/min_v, min_u/max_v, max_u/max_v]`.
    pub fn positions(&self, face: FaceDir) -> [[f32; 3]; 4] {
        let mut origin = [
            self.voxel[0] as f32,
            self.voxel[1] as f32,
            self.voxel[2] as f32,
        ];
        if face.is_positive() {
            origin[face.normal_axis()] += 1.0;
        }

        let u_axis = face.u_axis();
        let v_axis = face.v_axis();
        let w = self.width as f32;
        let h = self.height as f32;

        let min_u_min_v = origin;
        let mut max_u_min_v = origin;
        max_u_min_v[u_axis] += w;
        let mut min_u_max_v = origin;
        min_u_max_v[v_axis] += h;
        let mut max_u_max_v = origin;
        max_u_max_v[u_axis] += w;
        max_u_max_v[v_axis] += h;

        [min_u_min_v, max_u_min_v, min_u_max_v, max_u_max_v]
    }

    /// Returns the two-triangle index list for the quad with winding chosen
    /// so the outward normal points in [`FaceDir::normal`]'s direction.
    pub const fn indices(base_index: u32, face: FaceDir) -> [u32; 6] {
        // Which faces wind counter-clockwise was chosen to match
        // block-mesh-rs's output so Bevy's default backface culling keeps the
        // same set of visible triangles as before.
        let ccw = matches!(face, FaceDir::XNeg | FaceDir::YPos | FaceDir::ZPos);
        if ccw {
            [
                base_index,
                base_index + 1,
                base_index + 2,
                base_index + 1,
                base_index + 3,
                base_index + 2,
            ]
        } else {
            [
                base_index,
                base_index + 2,
                base_index + 1,
                base_index + 1,
                base_index + 2,
                base_index + 3,
            ]
        }
    }

    /// UV coordinates for each vertex (pre-atlas), in the same order as
    /// [`Quad::positions`]. Width/height are baked in so textures repeat
    /// across merged quads.
    pub const fn uvs(&self, face: FaceDir, flip_v: bool) -> [[f32; 2]; 4] {
        // Positive faces flip U to keep texture orientation consistent with
        // negative faces sharing the same axis.
        let flip_u = face.is_positive();
        let w = self.width as f32;
        let h = self.height as f32;
        match (flip_u, flip_v) {
            (false, false) => [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]],
            (true, false) => [[w, 0.0], [0.0, 0.0], [w, h], [0.0, h]],
            (false, true) => [[0.0, h], [w, h], [0.0, 0.0], [w, 0.0]],
            (true, true) => [[w, h], [0.0, h], [w, 0.0], [0.0, 0.0]],
        }
    }
}
