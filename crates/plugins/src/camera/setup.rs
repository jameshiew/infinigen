use bevy::core_pipeline::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasing;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;
use bevy::render::view::GpuCulling;
use bevy_flycam::FlyCam;

#[derive(Resource)]
pub struct CameraSettings {
    pub zoom_level: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotation_w: f32,
    pub wx: f32,
    pub wy: f32,
    pub wz: f32,
}

pub fn setup(mut commands: Commands, settings: Res<CameraSettings>) {
    let zoom = 2f32.powf(settings.zoom_level);
    let mut transform =
        Transform::from_xyz(settings.wx * zoom, settings.wy * zoom, settings.wz * zoom);
    transform.rotation.x = settings.rotation_x;
    transform.rotation.y = settings.rotation_y;
    transform.rotation.z = settings.rotation_z;
    transform.rotation.w = settings.rotation_w;
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
        GpuCulling,
    ));
}
