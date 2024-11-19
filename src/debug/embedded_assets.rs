use std::path::PathBuf;

use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
};

pub struct EmbeddedAssetsPlugin;

impl Plugin for EmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        let mod_file: PathBuf = file!().parse().unwrap();
        let mod_dir = mod_file.parent().unwrap();

        embedded_asset!(app, mod_dir, "shaders/line_material.wgsl");
    }
}
