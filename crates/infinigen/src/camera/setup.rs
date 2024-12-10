use bevy::core_pipeline::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasing;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;
use bevy_flycam::FlyCam;

use crate::settings::Config;

pub fn setup(mut commands: Commands, config: Res<Config>) {
    let zoom = 2f32.powf(config.zoom_level as f32);
    let mut transform = Transform::from_xyz(config.wx * zoom, config.wy * zoom, config.wz * zoom);
    transform.rotation.x = config.rotation_x;
    transform.rotation.y = config.rotation_y;
    transform.rotation.z = config.rotation_z;
    transform.rotation.w = config.rotation_w;
    transform.rotation = transform.rotation.normalize();
    dbg!(transform.rotation);
    commands.spawn((
        Name::new("Camera"),
        FlyCam,
        transform,
        Camera3d::default(),
        ContrastAdaptiveSharpening::default(),
        ScreenSpaceAmbientOcclusion::default(),
        TemporalAntiAliasing::default(),
        Msaa::Off,
    ));
}
