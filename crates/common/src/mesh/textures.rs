use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::world::ChunkBlockId;

const TEXTURE_SIZE: usize = 64;

// translate UVs for a texture atlas given raw coordinates
pub fn to_tex_coords_raw(
    mut uvs: [[f32; 2]; 4],
    texture_start: [usize; 2], // the top left corner of the texture in the atlas
    texture_atlas_size: [usize; 2], // the (width, height) of the texture atlas
) -> [[f32; 2]; 4] {
    for uv in &mut uvs {
        uv[0] *= TEXTURE_SIZE as f32;
        uv[1] *= TEXTURE_SIZE as f32;
        uv[0] += texture_start[0] as f32;
        uv[1] += texture_start[1] as f32;
        uv[0] /= texture_atlas_size[0] as f32;
        uv[1] /= texture_atlas_size[1] as f32;
    }
    uvs
}

#[derive(Default, Debug, Clone)]
pub struct TextureMap {
    /// Size of the texture atlas containing the textures.
    pub size: [usize; 2],
    /// Top left corner of texture in atlas, indexed by [`Face`].
    appearance: FxHashMap<ChunkBlockId, [FaceAppearance; 6]>,
}

#[derive(Debug, Clone, Copy)]
pub enum FaceAppearance {
    Texture { coords: [usize; 2] },
    Color { r: f32, g: f32, b: f32, a: f32 },
}

pub enum FaceAppearanceTransformed {
    Texture { coords: [[f32; 2]; 4] },
    Color { r: f32, g: f32, b: f32, a: f32 },
}

impl TextureMap {
    pub fn add(&mut self, id: ChunkBlockId, appearance: [FaceAppearance; 6]) {
        tracing::debug!(?id, ?appearance, "Recording appearance for block");
        self.appearance.insert(id, appearance);
    }

    pub fn get(
        &self,
        id: &ChunkBlockId,
        face: Face,
        uvs: [[f32; 2]; 4],
    ) -> Option<FaceAppearanceTransformed> {
        let appearances = self.appearance.get(id)?;
        match appearances[face as usize] {
            FaceAppearance::Texture { coords } => Some(FaceAppearanceTransformed::Texture {
                coords: to_tex_coords_raw(uvs, coords, self.size),
            }),
            FaceAppearance::Color { r, g, b, a } => {
                Some(FaceAppearanceTransformed::Color { r, g, b, a })
            }
        }
    }

    pub fn to_tex_coords(
        &self,
        id: &ChunkBlockId,
        face: Face,
        uvs: [[f32; 2]; 4],
    ) -> Option<[[f32; 2]; 4]> {
        let tidx = match self.appearance.get(id) {
            Some(faces) => faces[face as usize],
            None => {
                return None;
            }
        };
        match tidx {
            FaceAppearance::Texture { coords } => Some(to_tex_coords_raw(uvs, coords, self.size)),
            FaceAppearance::Color { .. } => None,
        }
    }

    pub fn to_color(&self, id: &ChunkBlockId, face: Face) -> Option<[f32; 4]> {
        let tidx = match self.appearance.get(id) {
            Some(faces) => faces[face as usize],
            None => {
                return None;
            }
        };
        match tidx {
            FaceAppearance::Texture { .. } => None,
            FaceAppearance::Color { r, g, b, a } => Some([r, g, b, a]),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd, EnumIter,
)]
pub enum Face {
    Top = 0,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Face {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }

    pub fn is_side(&self) -> bool {
        !matches!(self, Self::Top | Self::Bottom)
    }
}
