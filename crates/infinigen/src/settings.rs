use config_source::ConfigSource;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HORIZONTAL_VIEW_DISTANCE: u64 = 8;
pub const DEFAULT_VERTICAL_VIEW_DISTANCE: u64 = 8;

#[derive(Debug, Clone, Serialize, Deserialize, ConfigSource)]
pub struct AppSettings {
    pub hview_distance: u64,
    pub vview_distance: u64,
    pub world: String,

    #[serde(default)]
    pub zoom_level: i8,
    #[serde(default)]
    pub wx: f64,
    #[serde(default)]
    pub wy: f64,
    #[serde(default)]
    pub wz: f64,

    #[serde(default)]
    pub rotation_x: f64,
    #[serde(default)]
    pub rotation_y: f64,
    #[serde(default)]
    pub rotation_z: f64,
    #[serde(default)]
    pub rotation_w: f64,

    #[serde(default)]
    pub seed: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hview_distance: DEFAULT_HORIZONTAL_VIEW_DISTANCE,
            vview_distance: DEFAULT_VERTICAL_VIEW_DISTANCE,
            world: "MountainIslands".to_string(),
            zoom_level: 0,
            wx: -1283.,
            wy: 140.,
            wz: -1752.,
            rotation_x: -0.08,
            rotation_y: -0.9,
            rotation_z: -0.4,
            rotation_w: 0.18,
            seed: 0,
        }
    }
}
