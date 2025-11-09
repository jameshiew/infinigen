use serde::{Deserialize, Serialize};

const fn default_horizontal_view_distance() -> u64 {
    8
}

const fn default_vertical_view_distance() -> u64 {
    8
}

fn default_world() -> String {
    "MountainIslands".to_string() // TODO: remove this implicit dependency on infinigen_extras crate
}

const fn default_wx() -> f64 {
    -1283.
}

const fn default_wy() -> f64 {
    140.
}

const fn default_wz() -> f64 {
    -1752.
}

const fn default_target_x() -> f64 {
    -1300.
}

const fn default_target_y() -> f64 {
    6.
}

const fn default_target_z() -> f64 {
    -1615.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_horizontal_view_distance")]
    pub horizontal_view_distance: u64,
    #[serde(default = "default_vertical_view_distance")]
    pub vertical_view_distance: u64,
    #[serde(default = "default_world")]
    pub world: String,

    #[serde(default)]
    pub zoom_level: i8,
    #[serde(default = "default_wx")]
    pub wx: f64,
    #[serde(default = "default_wy")]
    pub wy: f64,
    #[serde(default = "default_wz")]
    pub wz: f64,

    #[serde(default = "default_target_x")]
    pub target_x: f64,
    #[serde(default = "default_target_y")]
    pub target_y: f64,
    #[serde(default = "default_target_z")]
    pub target_z: f64,

    #[serde(default)]
    pub seed: u64,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_config_parse() -> Result<()> {
        let _settings: AppSettings = config::Config::builder()
            .add_source(config::File::with_name("../../infinigen.config.yml"))
            .build()?
            .try_deserialize()?;
        Ok(())
    }
}
