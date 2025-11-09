#[cfg(not(target_arch = "wasm32"))]
use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;
use bevy::render::experimental::occlusion_culling::OcclusionCulling;
use bevy::render::view::Hdr;

use super::FpsController;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
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
    let zoom = (settings.zoom_level as f32).exp2();

    // Calculate initial position and rotation
    let eye_pos = Vec3::new(settings.wx * zoom, settings.wy * zoom, settings.wz * zoom);
    let target_pos = Vec3::new(
        settings.target_x * zoom,
        settings.target_y * zoom,
        settings.target_z * zoom,
    );

    // Calculate initial yaw and pitch from eye to target
    let direction = (target_pos - eye_pos).normalize();
    let yaw = direction.z.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
    let pitch = direction.y.asin();

    let controller = FpsController {
        yaw,
        pitch,
        ..FpsController::default()
    };

    let rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);

    commands.spawn((
        Name::new("Camera"),
        controller,
        Camera::default(),
        Hdr,
        Camera3d::default(),
        Transform::from_translation(eye_pos).with_rotation(rotation),
        #[cfg(not(target_arch = "wasm32"))]
        ContrastAdaptiveSharpening::default(),
        DepthPrepass,
        OcclusionCulling,
        ScreenSpaceAmbientOcclusion::default(),
        TemporalAntiAliasing::default(),
        Msaa::Off,
    ));
}
