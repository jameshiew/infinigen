use ahash::AHashMap;
use linearize::{Linearize, StaticCopyMap};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::world::MappedBlockID;

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
pub struct BlockAppearances {
    pub size: [usize; 2],
    appearance: AHashMap<MappedBlockID, StaticCopyMap<Face, FaceAppearance>>,
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

impl BlockAppearances {
    pub fn add(&mut self, id: MappedBlockID, appearance: StaticCopyMap<Face, FaceAppearance>) {
        tracing::debug!(?id, ?appearance, "Recording appearance for block");
        self.appearance.insert(id, appearance);
    }

    pub fn get(
        &self,
        id: &MappedBlockID,
        face: Face,
        uvs: [[f32; 2]; 4],
    ) -> Option<FaceAppearanceTransformed> {
        let appearances = self.appearance.get(id)?;
        match appearances[face] {
            FaceAppearance::Texture { coords } => Some(FaceAppearanceTransformed::Texture {
                coords: to_tex_coords_raw(uvs, coords, self.size),
            }),
            FaceAppearance::Color { r, g, b, a } => {
                Some(FaceAppearanceTransformed::Color { r, g, b, a })
            }
        }
    }

    pub fn to_color(&self, id: &MappedBlockID, face: Face) -> Option<[f32; 4]> {
        let appearances = self.appearance.get(id)?;
        match appearances[face] {
            FaceAppearance::Texture { .. } => None,
            FaceAppearance::Color { r, g, b, a } => Some([r, g, b, a]),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    EnumIter,
    Linearize,
)]
#[linearize(const)]
pub enum Face {
    Top = 0,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}
