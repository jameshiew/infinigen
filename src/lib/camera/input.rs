//! adapted from bevy_flycam
use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

/// Key configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::W,
            move_backward: KeyCode::S,
            move_left: KeyCode::A,
            move_right: KeyCode::D,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::LShift,
        }
    }
}

#[derive(Resource, Default)]
pub struct InputState {
    mouse_motion: ManualEventReader<MouseMotion>,
}

pub fn keyboard(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<(&mut Transform, &super::settings::Settings)>,
) {
    let window = primary_window.get_single().unwrap();

    for (mut transform, camera) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

        for key in keys.get_pressed() {
            match window.cursor.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    let key = *key;
                    if key == key_bindings.move_forward {
                        velocity += forward;
                    } else if key == key_bindings.move_backward {
                        velocity -= forward;
                    } else if key == key_bindings.move_left {
                        velocity -= right;
                    } else if key == key_bindings.move_right {
                        velocity += right;
                    } else if key == key_bindings.move_ascend {
                        velocity += Vec3::Y;
                    } else if key == key_bindings.move_descend {
                        velocity -= Vec3::Y;
                    }
                }
            }

            velocity = velocity.normalize_or_zero();
            transform.translation += velocity * time.delta_seconds() * camera.speed;
        }
    }
}

const SENSITIVITY: f32 = 0.0001;

pub fn mouse(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let window = primary_window.get_single().unwrap();
    for mut transform in query.iter_mut() {
        for ev in state.mouse_motion.iter(&motion) {
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            match window.cursor.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.height().min(window.width());
                    pitch -= (SENSITIVITY * ev.delta.y * window_scale).to_radians();
                    yaw -= (SENSITIVITY * ev.delta.x * window_scale).to_radians();
                }
            }

            pitch = pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}
