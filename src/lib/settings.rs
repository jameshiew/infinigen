use std::path::PathBuf;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::common::chunks::CHUNK_SIZE_F32;
use crate::extras::worldgen::WorldGenTypes;

pub const DEFAULT_HORIZONTAL_VIEW_DISTANCE: usize = 3;
pub const DEFAULT_VERTICAL_VIEW_DISTANCE: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Config {
    pub hview_distance: usize,
    pub vview_distance: usize,
    pub world: WorldGenTypes,
    pub save_dir: Option<PathBuf>,

    #[serde(default)]
    pub zoom_level: i8,
    #[serde(default)]
    pub wx: f32,
    #[serde(default)]
    pub wy: f32,
    #[serde(default)]
    pub wz: f32,

    #[serde(default)]
    pub rotation_x: f32,
    #[serde(default)]
    pub rotation_y: f32,
    #[serde(default)]
    pub rotation_z: f32,
    #[serde(default)]
    pub rotation_w: f32,
}

impl Default for Config {
    fn default() -> Self {
        let initial_x = CHUNK_SIZE_F32 / 2.;
        let initial_z = CHUNK_SIZE_F32 / 2.;
        let initial_height = CHUNK_SIZE_F32 / 2.;
        Self {
            hview_distance: DEFAULT_HORIZONTAL_VIEW_DISTANCE,
            vview_distance: DEFAULT_VERTICAL_VIEW_DISTANCE,
            world: WorldGenTypes::default(),
            save_dir: None,
            zoom_level: 0,
            wx: initial_x,
            wy: initial_height,
            wz: initial_z,
            rotation_x: 0.,
            rotation_y: 0.,
            rotation_z: 0.,
            rotation_w: 1.,
        }
    }
}
