use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use leafwing_input_manager::prelude::*;

use crate::AppState;

pub mod events;
pub mod setup;

/// First-person camera controller component
#[derive(Component, Debug)]
pub struct FpsController {
    /// Whether camera controls are enabled
    pub enabled: bool,
    /// Mouse sensitivity for looking around (radians per pixel)
    pub mouse_sensitivity: f32,
    /// Movement speed in units per second
    pub movement_speed: f32,
    /// Current pitch rotation (up/down angle in radians)
    pub pitch: f32,
    /// Current yaw rotation (left/right angle in radians)
    pub yaw: f32,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse_sensitivity: 0.002,
            movement_speed: 30.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

/// Component that tracks where the camera is looking at (for debug purposes)
#[derive(Component, Debug)]
pub struct CameraTarget {
    pub target: Vec3,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self { target: Vec3::ZERO }
    }
}

/// Camera movement and control actions
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum CameraAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
}

/// Set up camera input actions
fn setup_actions(mut commands: Commands) {
    let mut input_map = InputMap::new([
        (CameraAction::MoveForward, KeyCode::KeyW),
        (CameraAction::MoveBackward, KeyCode::KeyS),
        (CameraAction::MoveLeft, KeyCode::KeyA),
        (CameraAction::MoveRight, KeyCode::KeyD),
        (CameraAction::MoveUp, KeyCode::Space),
    ]);
    // Support both shift keys for moving down
    input_map.insert(CameraAction::MoveDown, KeyCode::ShiftLeft);
    input_map.insert(CameraAction::MoveDown, KeyCode::ShiftRight);

    commands.spawn(input_map);
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("Initializing camera plugin");
        app.add_plugins(InputManagerPlugin::<CameraAction>::default())
            .add_systems(
                OnEnter(AppState::InitializingWorld),
                (setup::setup, setup_actions),
            )
            .add_systems(
                Update,
                (
                    update_camera_look,
                    update_camera_movement,
                    update_camera_target,
                    events::handle_camera_events,
                )
                    .chain()
                    .run_if(in_state(AppState::MainGame)),
            )
            .add_event::<events::CameraEvent>();
    }
}

/// System to handle mouse look for the camera
fn update_camera_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut FpsController, &mut Transform), With<Camera>>,
    windows: Single<&Window>,
) {
    let Some((mut controller, mut transform)) = query.iter_mut().next() else {
        return;
    };

    if !controller.enabled {
        return;
    }

    // Check if cursor is grabbed
    if windows.cursor_options.grab_mode != CursorGrabMode::Locked {
        return;
    }

    // Accumulate mouse motion
    let mut delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        delta += motion.delta;
    }

    if delta.length_squared() > 0.0 {
        // Update yaw (left/right) and pitch (up/down)
        controller.yaw -= delta.x * controller.mouse_sensitivity;
        controller.pitch -= delta.y * controller.mouse_sensitivity;

        // Clamp pitch to prevent camera flipping
        controller.pitch = controller.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.001,
            std::f32::consts::FRAC_PI_2 - 0.001,
        );

        // Apply rotation to transform
        transform.rotation = Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
    }
}

/// System to handle WASD + space/shift movement
fn update_camera_movement(
    time: Res<Time>,
    action_state: Single<&ActionState<CameraAction>>,
    mut query: Query<(&FpsController, &mut Transform), With<Camera>>,
) {
    let Some((controller, mut transform)) = query.iter_mut().next() else {
        return;
    };

    if !controller.enabled {
        return;
    }

    let mut velocity = Vec3::ZERO;
    let forward = transform.forward();
    let right = transform.right();

    // WASD movement (horizontal plane for forward/back, actual right vector for strafing)
    if action_state.pressed(&CameraAction::MoveForward) {
        velocity += *forward;
    }
    if action_state.pressed(&CameraAction::MoveBackward) {
        velocity -= *forward;
    }
    if action_state.pressed(&CameraAction::MoveLeft) {
        velocity -= *right;
    }
    if action_state.pressed(&CameraAction::MoveRight) {
        velocity += *right;
    }

    // Vertical movement (fly up/down)
    if action_state.pressed(&CameraAction::MoveUp) {
        velocity += Vec3::Y;
    }
    if action_state.pressed(&CameraAction::MoveDown) {
        velocity -= Vec3::Y;
    }

    // Normalize to prevent faster diagonal movement, then apply speed
    if velocity.length_squared() > 0.0 {
        velocity = velocity.normalize() * controller.movement_speed * time.delta_secs();
        transform.translation += velocity;
    }
}

/// System to update the camera target position (what the camera is looking at)
fn update_camera_target(mut query: Query<(&Transform, &mut CameraTarget), With<Camera>>) {
    let Some((transform, mut target)) = query.iter_mut().next() else {
        return;
    };

    // Calculate a point some distance in front of the camera
    const LOOK_DISTANCE: f32 = 10.0;
    target.target = transform.translation + *transform.forward() * LOOK_DISTANCE;
}
