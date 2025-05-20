use bevy::core_pipeline::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasing;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;
use smooth_bevy_cameras::controllers::fps::{FpsCameraBundle, FpsCameraController};

#[derive(Resource)]
pub struct CameraSettings {
    pub zoom_level: i8,
    pub wx: f32,
    pub wy: f32,
    pub wz: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
}

pub fn setup(mut commands: Commands, settings: Res<CameraSettings>) {
    let zoom = 2f32.powf(settings.zoom_level as f32);

    let eye = Transform::from_xyz(settings.wx * zoom, settings.wy * zoom, settings.wz * zoom);
    let target = Transform::from_xyz(
        settings.target_x * zoom,
        settings.target_y * zoom,
        settings.target_z * zoom,
    );
    commands.spawn((
        Name::new("Camera"),
        FpsCameraBundle::new(
            FpsCameraController {
                enabled: true,
                mouse_rotate_sensitivity: Vec2::splat(0.2),
                translate_sensitivity: 30.0,
                smoothing_weight: 0.7,
            },
            eye.translation,
            target.translation,
            Vec3::Y,
        ),
        Camera {
            hdr: true,
            ..Camera::default()
        },
        Camera3d::default(),
        ContrastAdaptiveSharpening::default(),
        ScreenSpaceAmbientOcclusion::default(),
        TemporalAntiAliasing::default(),
        Msaa::Off,
    ));
}
