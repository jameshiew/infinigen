use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasing, pbr::ScreenSpaceAmbientOcclusion,
    prelude::*, utils::default,
};

use crate::{scene::FAR, settings::Config};

#[derive(Component)]
pub struct Settings {
    pub speed: f32,
}

pub const DEFAULT_SPEED: f32 = 30.;

impl Default for Settings {
    fn default() -> Self {
        Self {
            speed: DEFAULT_SPEED,
        }
    }
}

pub fn setup(mut commands: Commands, config: Res<Config>) {
    let zoom = 2f32.powf(config.zoom_level as f32);
    let mut transform = Transform::from_xyz(config.wx * zoom, config.wy * zoom, config.wz * zoom);
    transform.rotation.x = config.rotation_x;
    transform.rotation.y = config.rotation_y;
    transform.rotation.z = config.rotation_z;
    transform.rotation.w = config.rotation_w;
    transform.rotation = transform.rotation.normalize();
    dbg!(transform.rotation);
    commands
        .spawn((
            Name::new("Camera"),
            Projection::Perspective(PerspectiveProjection {
                far: FAR,
                ..Default::default()
            }),
            transform,
            Camera {
                hdr: true,
                ..default()
            },
            Camera3d::default(),
            Settings::default(),
        ))
        .insert(ScreenSpaceAmbientOcclusion::default())
        .insert(TemporalAntiAliasing::default())
        .insert(Msaa::Off);
}
