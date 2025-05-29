use std::f32::consts::PI;

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use infinigen_common::chunks::CHUNK_SIZE_F32;

use super::{FAR, SceneSettings, SceneView, SceneZoom};

pub const SKY_COLOR: Color = Color::srgb(0.47, 0.66, 1.);

pub fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight::default());
    commands.insert_resource(ClearColor(SKY_COLOR));

    commands.spawn((
        Name::new("Global lighting"),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, FAR / 8., 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: CHUNK_SIZE_F32 * 8.,
            maximum_distance: FAR,
            ..default()
        }
        .build(),
    ));
}

pub fn setup(
    mut scene_view: ResMut<SceneView>,
    mut scene_zoom: ResMut<SceneZoom>,
    settings: Res<SceneSettings>,
) {
    scene_view.hview_distance = settings.hview_distance;
    scene_view.vview_distance = settings.vview_distance;
    scene_zoom.prev_zoom_level = settings.zoom_level;
    scene_zoom.zoom_level = settings.zoom_level;
}
