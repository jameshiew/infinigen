use std::path::PathBuf;

use bevy::prelude::{default, Resource};
use serde::{Deserialize, Serialize};

use crate::extras::worldgen::WorldGenTypes;

pub const DEFAULT_HORIZONTAL_VIEW_DISTANCE: usize = 8;
pub const DEFAULT_VERTICAL_VIEW_DISTANCE: usize = 8;

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
        Self {
            hview_distance: DEFAULT_HORIZONTAL_VIEW_DISTANCE,
            vview_distance: DEFAULT_VERTICAL_VIEW_DISTANCE,
            world: WorldGenTypes::default(),
            save_dir: None,
            zoom_level: 0,
            wx: -1283.,
            wy: 140.,
            wz: -1752.,
            rotation_x: -0.08,
            rotation_y: -0.9,
            rotation_z: -0.4,
            rotation_w: 0.18,
        }
    }
}
